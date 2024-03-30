[![Crates.io](https://img.shields.io/crates/l/tasklib?style=plastic)](https://crates.io/crates/tasklib)
[![Crates.io](https://img.shields.io/crates/v/tasklib?style=plastic)](https://crates.io/crates/tasklib)
[![Docs](https://img.shields.io/badge/docs-latest-green?style=plastic)](https://docs.rs/tasklib/latest/tasklib/index.html)

# Description

Library used to interact with Taskwarrior in Rust.

# Usage

Add this crate to your `Cargo.toml` file, or use `cargo add tasklib`.

```toml
[dependencies]
tasklib = "0.3"
```

Here is a minimal example.

```rust
use tasklib::Task;

let json = r#"
{
  "id": 0,
  "description": "Task to do",
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
let task: Task = Task::from(json);
// Getting a String from your Serialized Task
let task_str: String = task.into();
```
