[![Crates.io](https://img.shields.io/crates/l/tasklib?style=plastic)](https://crates.io/crates/tasklib)
[![Crates.io](https://img.shields.io/crates/v/tasklib?style=plastic)](https://crates.io/crates/tasklib)
[![Docs](https://img.shields.io/badge/docs-latest-green?style=plastic)](https://docs.rs/tasklib/0.1.1/tasklib/)

# Description

Library to use Taskwarrior with Rust.

# Usage

Add this crate to your `Cargo.toml` file, or use `cargo add tasklib`.

```toml
[dependencies]
tasklib = "0.1.1"
```

Here is a simple serialization example.

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

let task: Task = serde_json::from_str(json).expect("valid json representing a task");

/* Task represents...
task = Task {
    id: 0,
    uuid: "d67fce70-c0b6-43c5-affc-a21e64567d40",
    description: "Task to do.",
    elapsed: Some(
        Duration {
            years: 0,
            months: 0,
            days: 0,
            hours: 2,
            minutes: 0,
            seconds: 0,
        },
    ),
    start: Some(
        2022-01-31T08:30:00Z,
    ),
    end: Some(
        2022-01-31T08:30:00Z,
    ),
    entry: 2022-01-31T08:30:00Z,
    modified: 2022-01-31T08:30:00Z,
    project: "Daily",
    status: Pending,
    tags: [
        "WORK",
    ],
    urgency: 9.91234,
    annotations: [],
}
*/
```
