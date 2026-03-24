[Latest Version]: https://img.shields.io/crates/v/rsz
[crates.io]: https://crates.io/crates/rsz
[License]: https://img.shields.io/crates/l/rsz
[gpl-3.0-or-later]: https://spdx.org/licenses/GPL-3.0-or-later.html

# About &emsp; [![Latest Version]][crates.io] [![License]][gpl-3.0-or-later]
Yet another RSZ document parser for data files from the RE Engine.

This project was created to drive the parsing step of [mhdb-wilds-data](https://github.com/LartTyler/mhdb-wilds-data).
Because of this, not all types supported by the RE Engine have been implemented yet. If you need support added for 
specific types, feel free to submit a pull request or open an issue requesting the types you need.

## Usage

```rust
use rsz::{Rsz, User, Value, LayoutMap};
use std::fs;
use std::fs::File;

fn main() -> Result<(), anyhow::Error> {
    // First, load an RSZ layout file, which contains the layouts of the data types
    // used by the engine.
    let layouts = fs::read_to_string("/path/to/rszexample.json")?;
    let layouts = LayoutMap::from_json(&layouts);

    // Next, parse the actual document. You can use `Rsz` as your entrypoint if you need
    // to handle multiple types of RSZ document.
    let doc = Rsz::load("/path/to/example.user.3", &layouts)?;
    
    match doc {
        Rsz::User(doc) => {
            println!("Parsed USR doc with {} root object(s)", doc.content.root_objects.len())
        }
        _ => unimplemented!("And so on..."),
    }

    // ... or, if you know the type of document ahead of time, you can skip the
    // intermediary type.
    let doc = User::load("/path/to/example.user.3", &layouts)?;

    // Root objects are stored in the `root_objects` property, and can be traversed to
    // begin walking the document.
    for root in &doc.content.root_objects {
        println!(
            "Found root object named {} with {} field(s)",
            root.name,
            root.fields.len()
        );
    }

    // Each `Field` in a root object contains a `Value` enum that tells you what type
    // is contained within that field, as well as its value.
    for field in &doc.root_objects[0].fields {
        println("Found field named {}", field.name);

        match field.value {
            Value::Boolean(value) => println!(">> Contained a boolean: {value:?}"),
            Value::U32(value) => println!(">> Contained a 32-bit unsigned integer: {value}"),
            Value::String(value) => {
                println!(
                    ">> Contained a string with length = {}: {}",
                    value.len(),
                    value,
                )
            }
            _ => unimplemented!("And so on..."),
        }
    }

    // For convenience, you can directly serialize the root objects on a document, e.g. to 
    // JSON to be used by another tool or application.
    let output_file = File::open("/path/to/output.json")?;
    serde_json::to_writer_pretty(output_file, doc.root_objects)?;

    Ok(())
}
```
