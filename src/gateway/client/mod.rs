//! A module for [`Client`] and supporting types.
//!
//! The [`Client`] contains information about a bot's token, as well as event handlers. Dispatching
//! events to handlers and starting sharded gateway connections is handled directly by the client.
//! In addition, the client automatically handles caching via the [`Cache`] struct.
//!
//! # Sharding
//!
//! If you do not require sharding - such as for a small bot - then use [`Client::start`]. If you
//! don't know what sharding is, refer to the [`sharding`] module documentation.
//!
//! There are a few methods of sharding available:
//! - [`Client::start_autosharded`]: retrieves the number of shards Discord recommends using from
//!   the API, and then automatically starts that number of shards.
//! - [`Client::start_shard`]: starts a single shard for use in the instance, handled by the
//!   instance of the Client. Use this if you only want 1 shard handled by this instance.
//! - [`Client::start_shards`]: starts all shards in this instance. This is best for when you want a
//!   completely shared State.
//! - [`Client::start_shard_range`]: start a range of shards within this instance. This should be
//!   used when you, for example, want to split 10 shards across 3 instances.
//!
//! Click [here][Client#examples] for an example on how to use a [`Client`].
//!
//! [`sharding`]: crate::gateway::sharding

mod context;
pub(crate) mod dispatch;
mod event_handler;

use std::num::NonZeroU16;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use serde_json::from_value;
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;
use tracing::{debug, warn};

pub use self::context::Context;
pub use self::event_handler::EventHandler;
use super::{
    DEFAULT_WAIT_BETWEEN_SHARD_START,
    ShardManager,
    ShardManagerOptions,
};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::all::{SecretString, BotGateway};

/// A builder implementing [`IntoFuture`] building a [`Client`] to interact with Discord.
#[must_use = "Builders do nothing unless they are awaited"]
pub struct ClientBuilder {
    token: SecretString,
    data: Option<Arc<dyn std::any::Any + Send + Sync>>,
    http: Arc<Http>,
    event_handler: Option<Arc<dyn EventHandler>>,
    wait_time_between_shard_start: Duration,
}

impl ClientBuilder {
    /// Construct a new builder to call methods on for the client construction. The `token` will
    /// automatically be prefixed "Bot " if not already.
    pub fn new(token: SecretString) -> Self {
        Self::new_with_http(token.clone(), Arc::new(Http::new(token)))
    }

    /// Construct a new builder with a [`Http`] instance to calls methods on for the client
    /// construction.
    pub fn new_with_http(token: SecretString, http: Arc<Http>) -> Self {
        Self {
            token,
            http,
            data: None,
            event_handler: None,
            wait_time_between_shard_start: DEFAULT_WAIT_BETWEEN_SHARD_START,
        }
    }

    /// Sets the global data type that can be accessed from [`Context::data`].
    pub fn data<D: std::any::Any + Send + Sync>(mut self, data: Arc<D>) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the time to wait between starting shards.
    ///
    /// This should only be used when using a gateway proxy, such as [Sandwich] or [Twilight Gateway
    /// Proxy], as otherwise this will lead to gateway disconnects if the shard start rate limit is
    /// not respected.
    ///
    /// [Sandwich]:  https://github.com/WelcomerTeam/Sandwich-Daemon
    /// [Twilight Gateway Proxy]: https://github.com/Gelbpunkt/gateway-proxy
    pub fn wait_time_between_shard_start(mut self, wait_time: Duration) -> Self {
        self.wait_time_between_shard_start = wait_time;
        self
    }

    /// Adds an event handler with multiple methods for each possible event.
    pub fn event_handler<H>(mut self, event_handler: impl Into<Arc<H>>) -> Self
    where
        H: EventHandler + 'static,
    {
        self.event_handler = Some(event_handler.into());
        self
    }

    /// Gets the added event handlers. See [`Self::event_handler`] for more info.
    #[must_use]
    pub fn get_event_handler(&self) -> Option<&Arc<dyn EventHandler>> {
        self.event_handler.as_ref()
    }
}

impl IntoFuture for ClientBuilder {
    type Output = Result<Client>;

    type IntoFuture = BoxFuture<'static, Result<Client>>;

    fn into_future(self) -> Self::IntoFuture {
        let data = self.data.unwrap_or(Arc::new(()));
        let http = self.http;

        Box::pin(async move {
            let json_resp = match http.get_bot_gateway().await {
                Ok(response) => response,
                Err(err) => {
                    warn!("HTTP request to get gateway URL failed: {err}");
                    return Err(err);
                },
            };

            let get_bot_gateway = from_value::<BotGateway>(json_resp);

            let (ws_url, shard_total, max_concurrency) = match get_bot_gateway {
                Ok(response) => (
                    Arc::from(response.url),
                    response.shards,
                    response.session_start_limit.max_concurrency,
                ),
                Err(err) => {
                    tracing::warn!("HTTP request to get gateway URL failed: {err}");
                    (Arc::from("wss://gateway.discord.gg"), NonZeroU16::MIN, NonZeroU16::MIN)
                },
            };

            let shard_manager = ShardManager::new(ShardManagerOptions {
                token: self.token,
                data: Arc::clone(&data),
                event_handler: self.event_handler,
                ws_url: Arc::clone(&ws_url),
                shard_total,
                max_concurrency,
                http: Arc::clone(&http),
                wait_time_between_shard_start: self.wait_time_between_shard_start,
            });

            let client = Client {
                data,
                shard_manager,
                ws_url,
                http,
            };
            Ok(client)
        })
    }
}

/// A high-level client that abstracts over the REST API as well as Discord's gateway.
///
/// It enables the user to start sending authenticated HTTP requests, plus also initialize a
/// WebSocket connection to the gateway through [`Shard`]s. Refer to the [documentation on using
/// sharding][super::sharding] for more information.
///
/// # Event Handlers
///
/// Event handlers can be configured. For example, the event handler will be dispatched to
/// whenever a [`Event::MessageCreate`] is received over the connection.
///
/// Note that you do not need to manually handle events, as they are handled internally and then
/// dispatched to your event handler.
///
/// [`Shard`]: crate::gateway::Shard
/// [`Event::MessageCreate`]: crate::model::event::Event::MessageCreate
pub struct Client {
    data: Arc<dyn std::any::Any + Send + Sync>,
    /// The shard manager for the client.
    ///
    /// This is the brains, managing shards (websocket connections) and bot lifecycle.
    pub shard_manager: ShardManager,
    /// URL that the client's shards will use to connect to the gateway.
    pub ws_url: Arc<str>,
    /// An HTTP client.
    pub http: Arc<Http>,
}

impl Client {
    pub fn builder(token: SecretString) -> ClientBuilder {
        ClientBuilder::new(token)
    }

    /// Fetches the data type provided to [`ClientBuilder::data`].
    ///
    /// See the documentation for [`Context::data`] for more information.
    #[must_use]
    pub fn data<Data: Send + Sync + 'static>(&self) -> Arc<Data> {
        self.try_data().expect("Client::data generic does not match ClientBuilder::data type")
    }

    /// Tries to fetch the data type provided to [`ClientBuilder::data`].
    #[must_use]
    pub fn try_data<Data: Send + Sync + 'static>(&self) -> Option<Arc<Data>> {
        Arc::clone(&self.data).downcast().ok()
    }

    /// Establish the connection and start listening for events.
    ///
    /// This will start receiving events in a loop and start dispatching the events to your
    /// registered handlers.
    ///
    /// Note that this should be used only for users and for bots which are in less than 2500
    /// guilds. If you have a reason for sharding and/or are in more than 2500 guilds, use one of
    /// these depending on your use case:
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information on effectively
    /// using sharding.
    ///
    /// # Examples
    ///
    /// Starting a Client with only 1 shard, out of 1 total:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = Token::from_env("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(token).await?;
    ///
    /// if let Err(why) = client.start().await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Gateway`] when all shards have shutdown due to an error.
    ///
    /// [gateway docs]: crate::gateway#sharding
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start(&mut self) -> Result<()> {
        self.start_connection(0, 0, NonZeroU16::MIN).await
    }

    /// Establish the connection(s) and start listening for events.
    ///
    /// This will start receiving events in a loop and start dispatching the events to your
    /// registered handlers.
    ///
    /// This will retrieve an automatically determined number of shards to use from the API -
    /// determined by Discord - and then open a number of shards equivalent to that amount.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information on effectively
    /// using sharding.
    ///
    /// # Examples
    ///
    /// Start as many shards as needed using autosharding:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = Token::from_env("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(token).await?;
    ///
    /// if let Err(why) = client.start_autosharded().await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Gateway`] when all shards have shutdown due to an error.
    ///
    /// [gateway docs]: crate::gateway#sharding
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start_autosharded(&mut self) -> Result<()> {
        let (end, total) = {
            let json_res = self.http.get_bot_gateway().await?;
            let res = from_value::<BotGateway>(json_res)?;
            (res.shards.get() - 1, res.shards)
        };

        self.start_connection(0, end, total).await
    }

    /// Establish a sharded connection and start listening for events.
    ///
    /// This will start receiving events and dispatch them to your registered handlers.
    ///
    /// This will create a single shard by ID. If using one shard per process, you will need to
    /// start other processes with the other shard IDs in some way.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information on effectively
    /// using sharding.
    ///
    /// # Examples
    ///
    /// Start shard 3 of 5:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = Token::from_env("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(token).await?;
    ///
    /// if let Err(why) = client.start_shard(3, 5).await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Start shard 0 of 1 (you may also be interested in [`Self::start`] or
    /// [`Self::start_autosharded`]):
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = Token::from_env("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(token).await?;
    ///
    /// if let Err(why) = client.start_shard(0, 1).await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Gateway`] when all shards have shutdown due to an error.
    ///
    /// [gateway docs]: crate::gateway#sharding
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start_shard(&mut self, shard: u16, shards: u16) -> Result<()> {
        self.start_connection(shard, shard, check_shard_total(shards)).await
    }

    /// Establish sharded connections and start listening for events.
    ///
    /// This will start receiving events and dispatch them to your registered handlers.
    ///
    /// This will create and handle all shards within this single process. If you only need to
    /// start a single shard within the process, or a range of shards, use [`Self::start_shard`] or
    /// [`Self::start_shard_range`], respectively.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information on effectively
    /// using sharding.
    ///
    /// # Examples
    ///
    /// Start all of 8 shards:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = Token::from_env("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(token).await?;
    ///
    /// if let Err(why) = client.start_shards(8).await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Gateway`] when all shards have shutdown due to an error.
    ///
    /// [Gateway docs]: crate::gateway#sharding
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start_shards(&mut self, total_shards: u16) -> Result<()> {
        self.start_connection(0, total_shards - 1, check_shard_total(total_shards)).await
    }

    /// Establish a range of sharded connections and start listening for events.
    ///
    /// This will start receiving events and dispatch them to your registered handlers.
    ///
    /// This will create and handle all shards within a given range within this single process. If
    /// you only need to start a single shard within the process, or all shards within the process,
    /// use [`Self::start_shard`] or [`Self::start_shards`], respectively.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information on effectively
    /// using sharding.
    ///
    /// # Examples
    ///
    /// For a bot using a total of 10 shards, initialize shards 4 through 7:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = Token::from_env("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(token).await?;
    ///
    /// if let Err(why) = client.start_shard_range(4..7, 10).await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Gateway`] when all shards have shutdown due to an error.
    ///
    /// [Gateway docs]: crate::gateway#sharding
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start_shard_range(&mut self, range: Range<u16>, total_shards: u16) -> Result<()> {
        self.start_connection(range.start, range.end, check_shard_total(total_shards)).await
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn start_connection(
        &mut self,
        start_shard: u16,
        end_shard: u16,
        total_shards: NonZeroU16,
    ) -> Result<()> {
        let init = end_shard - start_shard + 1;

        debug!("Initializing shard info: {} - {}/{}", start_shard, init, total_shards);

        self.shard_manager.run(start_shard, init, total_shards).await.map_err(Error::Gateway)
    }
}

fn check_shard_total(total_shards: u16) -> NonZeroU16 {
    NonZeroU16::new(total_shards).unwrap_or_else(|| {
        warn!("Invalid shard total provided ({total_shards}), defaulting to 1");
        NonZeroU16::MIN
    })
}
