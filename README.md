[![Crates.io](https://img.shields.io/crates/l/tasklib?style=plastic)](https://crates.io/crates/tasklib)
[![Crates.io](https://img.shields.io/crates/v/tasklib?style=plastic)](https://crates.io/crates/tasklib)
[![Docs](https://img.shields.io/badge/docs-latest-green?style=plastic)](https://docs.rs/tasklib/latest/tasklib/index.html)

# Description

Library used to interact with Taskwarrior in Rust.

# Usage

Add this crate to your `Cargo.toml` file, or use `cargo add tasklib`.

```toml
[dependencies]
tasklib = "0.4"
```

## Minimal Example

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

## From Standard Input

Example getting task from stdin and writing to stdout.

```rust
use tasklib::prelude::*;

// Getting a Task from stdin (example fails because it doesn't have actual JSON input)
let task: Task = Task::from_stdin().expect("read task from stdin as JSON");
// Writing a Task to stdout, as JSON
task.to_stdout().expect("write task to stdout as JSON");
```

## Command Line Arguments

Example getting command line arguments.

```rust no_run
use std::env;
use tasklib::prelude::*;

// Get the command line arguments.
let args: CliArguments = CliArguments::from(env::args());

args.hook(); // PathBuf::from("/home/.task/hooks/on-add.tsk")
args.api_version(); // ApiVersion::V2
args.arguments(); // String::from("task add Task to do.")
args.command(); // Command::Add
args.rc_file(); // PathBuf::from("/home/.taskrc")
args.data_location(); // PathBuf::from("/home/.task")
args.task_version(); // "3.4.2"
```

## Using `tasklib` Dependencies

Using the same dependencies as `tasklib`, such as `chrono`:

```rust
use tasklib::prelude::*;
use tasklib::chrono::offset::Utc;
use tasklib::chrono::DateTime;
```

## Task Builder

**Warning:** This API may be inadequate for some use cases, such as creating tasks not already found in a Taskwarrior database, as some fields are required by Taskwarrior for existing tasks (e.g. `uuid`).

Example creating a task using its builder:

```rust
use tasklib::prelude::*;
use tasklib::chrono::offset::Utc;
use tasklib::chrono::DateTime;

let task: Task = TaskBuilder::new()
    .description("Task to do.")
    .entry(DateTime::parse_from_rfc3339("2022-01-31T08:30:00Z").unwrap().with_timezone(&Utc))
    .modified(DateTime::parse_from_rfc3339("2022-01-31T08:30:00Z").unwrap().with_timezone(&Utc))
    .project("Daily")
    .start(DateTime::parse_from_rfc3339("2022-01-31T08:30:00Z").unwrap().with_timezone(&Utc))
    .status(Status::Pending)
    .uuid("d67fce70-c0b6-43c5-affc-a21e64567d40")
    .tags(vec!["WORK"])
    .urgency(9.91234)
    .parent("d67fce70-c0b6-43c5-affc-a21e64567d40")
    .build();
```
