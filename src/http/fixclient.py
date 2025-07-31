import time
with open("src/http/client.rs", "r") as file:
    code = file.read()

# Find all lines starting with `pub async fn` (along with some leading whitespace)
lines = code.split("\n")

in_async = False
data = ""
for i, line in enumerate(lines):
    if line.strip().startswith("pub async fn") and "<T:" not in line.strip() and "pub async fn request(&self" not in line.strip():
        in_async = True
        data = ""

    if in_async:
        data += line + "\n"
        if " -> Result<" in line:
            in_async = False

            # Check the next line for a self.wind call, if so, ignore it
            if i + 1 < len(lines) and "self.wind" in lines[i + 1]:
                in_async = False
                continue

            print(f"found line to modify: {data}")

            # Change the return type from Result<...> to ResultJson
            result_type = line.split(" -> ")[1]

            modified_line = line.replace(result_type, "ResultJson {")

            print("modified line:", modified_line)

            lines[i] = modified_line

with open("src/http/client_new.rs", "w") as file:
    file.write("\n".join(lines))