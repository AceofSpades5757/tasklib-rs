[![Crates.io](https://img.shields.io/crates/l/tasklib?style=plastic)](https://crates.io/crates/tasklib)
[![Crates.io](https://img.shields.io/crates/v/tasklib?style=plastic)](https://crates.io/crates/tasklib)
[![Docs](https://img.shields.io/badge/docs-latest-green?style=plastic)](https://docs.rs/tasklib/0.1.1/tasklib/)

# Description

Library to use Taskwarrior with Rust.

# Usage

Add this crate to your `Cargo.toml` file, or use `cargo add tasklib`.

```toml
[dependencies]
tasklib = "0.1"
```

Here is a minimal example.

```rust
use tasklib::Task;

let json = r#"
{
  "id": 0,
  "description": "Task to do.",
  "elapsed": "PT2H",
  "end": "20220131T083000Z",
  "entry": "20220131T083000Z",
  "modified": "20220131T083000Z",
  "project": "Daily",
  "start": "20220131T083000Z",
  "status": "pending",
  "uuid": "d67fce70-c0b6-43c5-affc-a21e64567d40",
  "tags": [
    "WORK"
  ],
  "urgency": 9.91234
}"#;

// Getting a Task from your input JSON string.
let task: Task = serde_json::from_str(json).expect("valid json parsed into a task");
// Getting a String from your Serialized Task
let task_str: String = serde_json::to_string(&task).expect("valid json string representing a task");
```
