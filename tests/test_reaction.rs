use serenity::model::channel::ReactionType;

#[test]
fn json_to_reaction_type() {
    let s = r#"{"name": "foo", "id": "1"}"#;
    let value = serde_json::from_str(s).unwrap();
    assert!(matches!(value, ReactionType::Custom { .. }));
    if let ReactionType::Custom {
        name, ..
    } = value
    {
        assert_eq!(name.as_deref(), Some("foo"));
    }

    let s = r#"{"name": null, "id": "1"}"#;
    let value = serde_json::from_str(s).unwrap();
    assert!(matches!(value, ReactionType::Custom { .. }));

    let s = r#"{"id": "1"}"#;
    let value = serde_json::from_str(s).unwrap();
    assert!(matches!(value, ReactionType::Custom { .. }));

    let s = r#"{"name": "foo"}"#;
    let value = serde_json::from_str(s).unwrap();
    assert!(matches!(value, ReactionType::Unicode(_)));
    if let ReactionType::Unicode(value) = value {
        assert_eq!(&*value, "foo");
    }

    let s = r#"{"name": null}"#;
    assert!(serde_json::from_str::<ReactionType>(s).is_err());
}
