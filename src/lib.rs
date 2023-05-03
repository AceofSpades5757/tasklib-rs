//! Minimal example using `tasklib`.
//!
//! ```rust
//! use tasklib::prelude::*;
//!
//! let json = r#"
//! {
//!   "id": 0,
//!   "description": "Task to do.",
//!   "end": "20220131T083000Z",
//!   "entry": "20220131T083000Z",
//!   "modified": "20220131T083000Z",
//!   "project": "Daily",
//!   "start": "20220131T083000Z",
//!   "status": "pending",
//!   "uuid": "d67fce70-c0b6-43c5-affc-a21e64567d40",
//!   "tags": [
//!     "WORK"
//!   ],
//!   "urgency": 9.91234
//! }"#;
//!
//! // Getting a Task from your input JSON string.
//! let task: Task = Task::from(json);
//! // Getting a String from your Serialized Task
//! let task_str: String = task.into();
//! ```
//!
//! Example getting task from stdin and writing to stdout.
//!
//! ```should_panic rust
//! use tasklib::prelude::*;
//!
//! // Getting a Task from stdin
//! let task: Task = Task::from_stdin().expect("read task from stdin as JSON");
//! // Writing a Task to stdout, as JSON
//! task.to_stdout().expect("write task to stdout as JSON");
//! ```

use std::collections::HashMap;
use std::fmt;
use std::io::{self, Read, Write};
use std::str::FromStr;
use std::string::ToString;
use uuid::Uuid;

use chrono::{self, offset::Utc, DateTime, NaiveDateTime};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use duration::Duration;
use udas::UdaValue;

mod duration;

const DATETIME_FORMAT: &str = "%Y%m%dT%H%M%SZ";

/// Taskwarrior str to DateTime<Utc> deserializer
///
/// str -> DateTime<Utc>
fn tw_str_to_dt_de<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    struct DateTimeStringVisitor;

    impl<'de> de::Visitor<'de> for DateTimeStringVisitor {
        type Value = DateTime<Utc>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containg datetime data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(DateTime::<Utc>::from_utc(
                NaiveDateTime::parse_from_str(v, DATETIME_FORMAT)
                    .expect("string turned into datetime"),
                Utc,
            ))
        }
    }
    deserializer.deserialize_any(DateTimeStringVisitor)
}

/// Taskwarrior str to Option<DateTime<Utc>> deserializer
///
/// str -> Option<DateTime<Utc>>
fn tw_str_to_dt_opt_de<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct DateTimeStringVisitor;

    impl<'de> de::Visitor<'de> for DateTimeStringVisitor {
        type Value = Option<DateTime<Utc>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containg datetime data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(DateTime::<Utc>::from_utc(
                NaiveDateTime::parse_from_str(v, DATETIME_FORMAT)
                    .expect("string turned into datetime"),
                Utc,
            )))
        }
    }
    deserializer.deserialize_any(DateTimeStringVisitor)
}

/// Taskwarrior str to DateTime<Utc> serializer
///
/// DateTime<Utc> -> String
fn tw_dt_to_str_se<S: Serializer>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&dt.format(&DATETIME_FORMAT).to_string())
}

/// Taskwarrior str to Option<DateTime<Utc>> serializer
///
/// Option<DateTime<Utc>> -> String
fn tw_dt_to_str_opt_se<S: Serializer>(dt: &Option<DateTime<Utc>>, s: S) -> Result<S::Ok, S::Error> {
    match dt {
        Some(dt) => s.serialize_str(&dt.format(&DATETIME_FORMAT).to_string()),
        None => s.serialize_str(""),
    }
}

/// See all columns using `task columns` and `task _columns`.
///
/// UDAs will only deserialize to a string or numeric type. Durations and dates will be parsed to a string.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    /// Task ID
    ///
    /// This is the internal ID of the task, and is not the same as the UUID.
    ///
    /// This is temporary and may not exist for some tasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<usize>,
    uuid: Uuid,
    description: String,
    #[serde(
        serialize_with = "tw_dt_to_str_opt_se",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "tw_str_to_dt_opt_de",
        default
    )]
    start: Option<DateTime<Utc>>,
    #[serde(
        serialize_with = "tw_dt_to_str_opt_se",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "tw_str_to_dt_opt_de",
        default
    )]
    end: Option<DateTime<Utc>>,
    #[serde(
        serialize_with = "tw_dt_to_str_se",
        deserialize_with = "tw_str_to_dt_de"
    )]
    entry: DateTime<Utc>,
    #[serde(
        serialize_with = "tw_dt_to_str_opt_se",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "tw_str_to_dt_opt_de",
        default
    )]
    scheduled: Option<DateTime<Utc>>,
    #[serde(
        serialize_with = "tw_dt_to_str_opt_se",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "tw_str_to_dt_opt_de",
        default
    )]
    until: Option<DateTime<Utc>>,
    #[serde(
        serialize_with = "tw_dt_to_str_opt_se",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "tw_str_to_dt_opt_de",
        default
    )]
    wait: Option<DateTime<Utc>>,
    #[serde(
        serialize_with = "tw_dt_to_str_opt_se",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "tw_str_to_dt_opt_de",
        default
    )]
    due: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    depends: Vec<Uuid>,
    /// <https://taskwarrior.org/docs/commands/columns/>
    /// Type: numeric
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    imask: Option<i64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    mask: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<Uuid>,
    /// Used with recurance templates.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    recur: Option<Duration>,
    #[serde(
        serialize_with = "tw_dt_to_str_se",
        deserialize_with = "tw_str_to_dt_de"
    )]
    modified: DateTime<Utc>,
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    project: String,
    status: Status,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    urgency: Option<f64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    annotations: Vec<Annotation>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(flatten)]
    udas: HashMap<String, UdaValue>,
}

/// Getters (Immutable)
impl Task {
    pub fn id(&self) -> &Option<usize> {
        &self.id
    }
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn start(&self) -> Option<&DateTime<Utc>> {
        self.start.as_ref()
    }
    pub fn end(&self) -> Option<&DateTime<Utc>> {
        self.end.as_ref()
    }
    pub fn entry(&self) -> &DateTime<Utc> {
        &self.entry
    }
    pub fn modified(&self) -> &DateTime<Utc> {
        &self.modified
    }
    pub fn project(&self) -> &str {
        &self.project
    }
    pub fn status(&self) -> &Status {
        &self.status
    }
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
    pub fn urgency(&self) -> &Option<f64> {
        &self.urgency
    }
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
    pub fn udas(&self) -> &HashMap<String, UdaValue> {
        &self.udas
    }
}

/// Getters (Mutable)
impl Task {
    pub fn id_mut(&mut self) -> &mut Option<usize> {
        &mut self.id
    }
    pub fn uuid_mut(&mut self) -> &mut Uuid {
        &mut self.uuid
    }
    pub fn description_mut(&mut self) -> &mut String {
        &mut self.description
    }
    pub fn start_mut(&mut self) -> &mut Option<DateTime<Utc>> {
        &mut self.start
    }
    pub fn end_mut(&mut self) -> &mut Option<DateTime<Utc>> {
        &mut self.end
    }
    pub fn entry_mut(&mut self) -> &mut DateTime<Utc> {
        &mut self.entry
    }
    pub fn modified_mut(&mut self) -> &mut DateTime<Utc> {
        &mut self.modified
    }
    pub fn project_mut(&mut self) -> &mut String {
        &mut self.project
    }
    pub fn status_mut(&mut self) -> &mut Status {
        &mut self.status
    }
    pub fn tags_mut(&mut self) -> &mut Vec<String> {
        &mut self.tags
    }
    pub fn urgency_mut(&mut self) -> &mut Option<f64> {
        &mut self.urgency
    }
    pub fn annotations_mut(&mut self) -> &mut Vec<Annotation> {
        &mut self.annotations
    }
    pub fn udas_mut(&mut self) -> &mut HashMap<String, UdaValue> {
        &mut self.udas
    }
}

/// Constructors
impl Task {
    pub fn from_reader(reader: impl Read) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(reader)
    }
    pub fn from_stdin() -> Result<Self, serde_json::Error> {
        Self::from_reader(io::stdin())
    }
}

/// Conversion Methods
impl Task {
    /// Convert Task to JSON object.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    /// Convert Task to JSON string.
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    /// Write JSON representation of Task to handle.
    pub fn to_writer(&self, writer: impl Write) -> Result<(), serde_json::Error> {
        serde_json::to_writer(writer, self)
    }
    /// Write JSON representation of Task to stdout.
    pub fn to_stdout(&self) -> Result<(), serde_json::Error> {
        self.to_writer(io::stdout())
    }
}

/// ToString (JSON)
///
/// Uses JSON as this is the most common use case for converting a Task to a string.
impl ToString for Task {
    fn to_string(&self) -> String {
        self.to_json_string()
    }
}

impl FromStr for Task {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let task: Task = serde_json::from_str(s)?;
        Ok(task)
    }
}

impl From<Task> for String {
    fn from(task: Task) -> Self {
        serde_json::to_string(&task).expect("task turned into string")
    }
}

impl From<String> for Task {
    fn from(s: String) -> Self {
        Task::from_str(&s).expect("string turned into task")
    }
}

impl From<&str> for Task {
    fn from(s: &str) -> Self {
        Task::from_str(s).expect("string turned into task")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotation {
    #[serde(
        serialize_with = "tw_dt_to_str_se",
        deserialize_with = "tw_str_to_dt_de"
    )]
    entry: DateTime<Utc>,
    description: String,
}

// #[derive(Debug, Serialize)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Status {
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "recurring")]
    Recurring,
    #[serde(rename = "deleted")]
    Deleted,
}

#[derive(Debug, Default)]
pub struct TaskBuilder {
    id: Option<usize>,
    uuid: Option<Uuid>,
    description: Option<String>,
    entry: Option<DateTime<Utc>>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    modified: Option<DateTime<Utc>>,
    status: Option<Status>,
    tags: Option<Vec<String>>,
    annotations: Option<Vec<Annotation>>,
    priority: Option<String>,
    project: Option<String>,
    wait: Option<DateTime<Utc>>,
    due: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
    scheduled: Option<DateTime<Utc>>,
    recur: Option<Duration>,
    mask: Option<String>,
    imask: Option<i64>,
    parent: Option<Uuid>,
    depends: Option<Vec<Uuid>>,
    urgency: Option<f64>,
    udas: Option<HashMap<String, UdaValue>>,
}

impl TaskBuilder {
    pub fn id(mut self, id: usize) -> Self {
        self.id = Some(id);
        self
    }
    pub fn uuid(mut self, uuid: &str) -> Self {
        self.uuid = Some(Uuid::parse_str(uuid).expect("valid uuid"));
        self
    }
    pub fn description<T: ToString>(mut self, description: T) -> Self {
        self.description = Some(description.to_string());
        self
    }
    pub fn entry(mut self, entry: DateTime<Utc>) -> Self {
        self.entry = Some(entry);
        self
    }
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }
    pub fn modified(mut self, modified: DateTime<Utc>) -> Self {
        self.modified = Some(modified);
        self
    }
    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }
    pub fn tag(mut self, tag: String) -> Self {
        if let Some(tags) = &mut self.tags {
            tags.push(tag);
        } else {
            self.tags = Some(vec![tag]);
        }
        self
    }
    pub fn tags<T: ToString>(mut self, tags: Vec<T>) -> Self {
        if let Some(existing_tags) = &mut self.tags {
            existing_tags.extend(tags.into_iter().map(|t| t.to_string()));
        } else {
            self.tags = Some(tags.into_iter().map(|t| t.to_string()).collect());
        }
        self
    }
    pub fn annotations(mut self, annotations: Vec<Annotation>) -> Self {
        self.annotations = Some(annotations);
        self
    }
    pub fn priority(mut self, priority: String) -> Self {
        self.priority = Some(priority);
        self
    }
    pub fn project<T: ToString>(mut self, project: T) -> Self {
        self.project = Some(project.to_string());
        self
    }
    pub fn wait(mut self, wait: DateTime<Utc>) -> Self {
        self.wait = Some(wait);
        self
    }
    pub fn scheduled(mut self, scheduled: DateTime<Utc>) -> Self {
        self.scheduled = Some(scheduled);
        self
    }
    pub fn recur(mut self, recur: Duration) -> Self {
        self.recur = Some(recur);
        self
    }
    pub fn mask(mut self, mask: String) -> Self {
        self.mask = Some(mask);
        self
    }
    pub fn imask(mut self, imask: i64) -> Self {
        self.imask = Some(imask);
        self
    }
    pub fn parent(mut self, parent: &str) -> Self {
        self.parent = Some(Uuid::parse_str(parent).expect("valid uuid"));
        self
    }
    pub fn urgency(mut self, urgency: f64) -> Self {
        self.urgency = Some(urgency);
        self
    }
    pub fn uda(mut self, name: String, uda: UdaValue) -> Self {
        self.udas.get_or_insert_with(HashMap::new).insert(name, uda);
        self
    }
}

impl TaskBuilder {
    pub fn new() -> Self {
        TaskBuilder {
            ..Default::default()
        }
    }
    pub fn build(self) -> Task {
        Task {
            id: self.id,
            uuid: self.uuid.unwrap(),
            description: self.description.unwrap_or("".to_string()),
            entry: self.entry.unwrap_or(Utc::now()),
            start: self.start,
            end: self.end,
            modified: self.modified.unwrap(),
            status: self.status.unwrap(),
            tags: self.tags.unwrap_or(vec![]),
            annotations: self.annotations.unwrap_or(vec![]),
            project: self.project.unwrap_or("".to_string()),
            scheduled: self.scheduled,
            until: self.until,
            recur: self.recur,
            mask: self.mask,
            imask: self.imask,
            parent: self.parent,
            depends: self.depends.unwrap_or(vec![]),
            wait: self.wait,
            due: self.due,
            urgency: self.urgency,
            udas: self.udas.unwrap_or(HashMap::new()),
        }
    }
}

mod udas {

    use super::tw_dt_to_str_opt_se;
    use super::tw_dt_to_str_se;
    use super::tw_str_to_dt_de;
    use super::tw_str_to_dt_opt_de;
    use super::Duration;
    use chrono::{self, offset::Utc, DateTime};

    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub enum UdaValue {
        String(String),
        Numeric(f64),
        #[serde(serialize_with = "tw_dt_to_str_se")]
        Date(DateTime<Utc>),
        Duration(Duration),
    }

    impl<'de> serde::Deserialize<'de> for UdaValue {
        fn deserialize<D>(deserializer: D) -> Result<UdaValue, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct UdaValueVisitor;

            impl<'de> serde::de::Visitor<'de> for UdaValueVisitor {
                type Value = UdaValue;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a string or a number")
                }

                fn visit_str<E>(self, value: &str) -> Result<UdaValue, E>
                where
                    E: serde::de::Error,
                {
                    Ok(UdaValue::String(value.to_string()))
                }

                fn visit_i64<E>(self, value: i64) -> Result<UdaValue, E>
                where
                    E: serde::de::Error,
                {
                    Ok(UdaValue::Numeric(value as f64))
                }

                fn visit_u64<E>(self, value: u64) -> Result<UdaValue, E>
                where
                    E: serde::de::Error,
                {
                    Ok(UdaValue::Numeric(value as f64))
                }

                fn visit_f64<E>(self, value: f64) -> Result<UdaValue, E>
                where
                    E: serde::de::Error,
                {
                    Ok(UdaValue::Numeric(value))
                }
            }

            deserializer.deserialize_any(UdaValueVisitor)
        }
    }

    /// Implement tasklib::Duration into UdaValue
    impl From<Duration> for UdaValue {
        fn from(d: Duration) -> Self {
            UdaValue::Duration(d)
        }
    }

    /// Implement chrono::DateTime into UdaValue
    impl From<DateTime<Utc>> for UdaValue {
        fn from(d: DateTime<Utc>) -> Self {
            UdaValue::Date(d)
        }
    }

    /// Implement == against string
    impl PartialEq<String> for UdaValue {
        fn eq(&self, other: &String) -> bool {
            match self {
                UdaValue::String(s) => s == other,
                _ => false,
            }
        }
    }

    /// Implement == against &str
    impl PartialEq<str> for UdaValue {
        fn eq(&self, other: &str) -> bool {
            match self {
                UdaValue::String(s) => s == other,
                _ => false,
            }
        }
    }

    /// Implement == against f64
    impl PartialEq<f64> for UdaValue {
        fn eq(&self, other: &f64) -> bool {
            match self {
                UdaValue::Numeric(n) => n == other,
                _ => false,
            }
        }
    }

    /// Implement == against i64
    impl PartialEq<i64> for UdaValue {
        fn eq(&self, other: &i64) -> bool {
            match self {
                UdaValue::Numeric(n) => *n as i64 == *other,
                _ => false,
            }
        }
    }

    /// Implement == against DateTime<Utc>
    impl PartialEq<DateTime<Utc>> for UdaValue {
        fn eq(&self, other: &DateTime<Utc>) -> bool {
            match self {
                UdaValue::Date(d) => d == other,
                _ => false,
            }
        }
    }

    /// Implement == against Duration
    impl PartialEq<Duration> for UdaValue {
        fn eq(&self, other: &Duration) -> bool {
            match self {
                UdaValue::Duration(d) => d == other,
                _ => false,
            }
        }
    }

    /// Represents a Taskwarrior UDA
    ///
    /// <https://taskwarrior.org/docs/udas/>
    ///
    /// UDAs, at their core, have a name and a value. The name is a string and the value can be one
    /// of 4 types: string, numeric, date, or duration.
    ///
    /// Each can have a label, default, and/or coeeficient.
    ///
    /// The label defaults to the capitalized form of the name.
    ///
    /// A string type can have a list of values.
    //#[derive(Debug, Clone, PartialEq)]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum Uda {
        String {
            name: String,
            value: String,
            // defaults to ""
            label: String,
            default: String,
            values: Vec<String>,
            coefficient: Option<f32>,
        },
        Numeric {
            name: String,
            value: f64,
            // defaults to ""
            label: String,
            default: f64,
            coefficient: Option<f32>,
        },
        Date {
            name: String,
            #[serde(
                serialize_with = "tw_dt_to_str_se",
                deserialize_with = "tw_str_to_dt_de"
            )]
            value: DateTime<Utc>,
            // defaults to ""
            label: String,
            #[serde(
                serialize_with = "tw_dt_to_str_opt_se",
                skip_serializing_if = "Option::is_none",
                deserialize_with = "tw_str_to_dt_opt_de",
                default
            )]
            default: Option<DateTime<Utc>>,
            coefficient: Option<f32>,
        },
        Duration {
            name: String,
            value: Duration,
            // defaults to ""
            label: String,
            default: Option<Duration>,
            coefficient: Option<f32>,
        },
    }

    /// Manually implement Deserialize for Uda
    ///
    /// "elapsed": "2.0",   -> Uda::String { name: String::from("elapsed"), value: String::from("2.0"), .. }
    /// "elapsed": 2.0,     -> Uda::Numeric { name: String::from("elapsed"), value: 2.0, .. }
    /// "elapsed": "20220131T083000Z", -> Uda::Date { name: String::from("elapsed"), value: Utc.datetime_from_str("20220131T083000Z", "%Y%m%dT%H%M%SZ")
    /// "elapsed": "PT2H",  -> Uda::Duration { name: String::from("elapsed"), value: Duration::hours(2), .. }

    /// Allow Uda::String{ .. } to be compared to a string
    ///
    /// Uses the `value` field of the UDA
    impl PartialEq<String> for Uda {
        fn eq(&self, other: &String) -> bool {
            match self {
                Uda::String { value, .. } => value == other,
                _ => false,
            }
        }
    }

    impl PartialEq<str> for Uda {
        fn eq(&self, other: &str) -> bool {
            match self {
                Uda::String { value, .. } => value == other,
                _ => false,
            }
        }
    }

    impl Uda {
        // This type isn't yet implemented and may be deprecated
        #[allow(dead_code)]
        /// Get the type of the UDA as a string
        pub fn r#type(&self) -> String {
            match self {
                Uda::String { .. } => "string".to_string(),
                Uda::Numeric { .. } => "numeric".to_string(),
                Uda::Date { .. } => "date".to_string(),
                Uda::Duration { .. } => "duration".to_string(),
            }
        }
    }

    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Debug, Clone)]
    enum Type {
        /// May be provided a list of acceptable values, using the `uda.my_uda.values` key, which
        /// is set to a string of comma-separated values.
        ///
        /// e.g. `task config uda.size.values large,medium,small`
        String,
        /// Float
        Numeric,
        /// I'm using chrono's DateTime struct
        Date,
        Duration,
    }

    impl Type {
        fn to_string(&self) -> String {
            self.to_str().to_string()
        }
        fn to_str(&self) -> &str {
            match self {
                Type::String => "string",
                Type::Numeric => "numeric",
                Type::Date => "date",
                Type::Duration => "duration",
            }
        }
        fn from_str(s: &str) -> Result<Type, String> {
            match s {
                "string" => Ok(Type::String),
                "numeric" => Ok(Type::Numeric),
                "date" => Ok(Type::Date),
                "duration" => Ok(Type::Duration),
                _ => Err(format!("invalid type: {}", s)),
            }
        }
        fn from_string(s: String) -> Result<Type, String> {
            Type::from_str(&s)
        }
    }

    impl Serialize for Type {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_string())
        }
    }

    impl<'de> Deserialize<'de> for Type {
        fn deserialize<D>(deserializer: D) -> Result<Type, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Type::from_string(s).map_err(de::Error::custom)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn basic_uda() {
            let uda_1 = Uda::String {
                name: "my_uda".to_string(),
                value: "my_value".to_string(),
                label: "".to_string(),
                default: "my_default".to_string(),
                values: vec!["my_value".to_string(), "my_value_2".to_string()],
                coefficient: Some(1.0),
            };
            let uda_2 = Uda::String {
                name: "my_uda".to_string(),
                value: "my_value".to_string(),
                label: "".to_string(),
                default: "my_default".to_string(),
                values: vec!["my_value".to_string(), "my_value_2".to_string()],
                coefficient: Some(1.0),
            };

            assert_eq!(uda_1, uda_2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialize_durations_iso_8601() {
        assert_eq!("P3D".parse::<Duration>().unwrap(), Duration::days(3),);
        assert_eq!("P1000D".parse::<Duration>().unwrap(), Duration::days(1000),);
        assert_eq!("PT10M".parse::<Duration>().unwrap(), Duration::minutes(10),);
        assert_eq!("P10M".parse::<Duration>().unwrap(), Duration::months(10),);
        assert_eq!(
            "P2M3D".parse::<Duration>().unwrap(),
            Duration::months(2) + Duration::days(3)
        );
        assert_eq!("P1Y".parse::<Duration>().unwrap(), Duration::years(1),);
        assert_eq!(
            "P1Y3D".parse::<Duration>().unwrap(),
            Duration::years(1) + Duration::days(3)
        );
        assert_eq!("PT50S".parse::<Duration>().unwrap(), Duration::seconds(50));
        assert_eq!("PT40M".parse::<Duration>().unwrap(), Duration::minutes(40));
        assert_eq!(
            "PT40M50S".parse::<Duration>().unwrap(),
            Duration::minutes(40) + Duration::seconds(50)
        );
        assert_eq!(
            "PT12H40M50S".parse::<Duration>().unwrap(),
            Duration::hours(12) + Duration::minutes(40) + Duration::seconds(50)
        );
        assert_eq!(
            "P1Y2M3DT12H40M50S".parse::<Duration>().unwrap(),
            Duration::years(1)
                + Duration::months(2)
                + Duration::days(3)
                + Duration::hours(12)
                + Duration::minutes(40)
                + Duration::seconds(50)
        );
        assert_eq!(
            {
                let dur: Duration = "P1Y2M3DT12H40M50S".into();
                dur
            },
            Duration::years(1)
                + Duration::months(2)
                + Duration::days(3)
                + Duration::hours(12)
                + Duration::minutes(40)
                + Duration::seconds(50)
        );
        assert_eq!(
            {
                let dur: Duration = "P1Y2M3DT12H40M50S".to_string().into();
                dur
            },
            Duration::years(1)
                + Duration::months(2)
                + Duration::days(3)
                + Duration::hours(12)
                + Duration::minutes(40)
                + Duration::seconds(50)
        );
        assert_eq!(
            {
                let dur: Duration = Duration::from("P1Y2M3DT12H40M50S");
                dur
            },
            Duration::years(1)
                + Duration::months(2)
                + Duration::days(3)
                + Duration::hours(12)
                + Duration::minutes(40)
                + Duration::seconds(50)
        );
        assert_eq!(
            {
                let dur: Duration = Duration::from("P1Y2M3DT12H40M50S".to_string());
                dur
            },
            Duration::years(1)
                + Duration::months(2)
                + Duration::days(3)
                + Duration::hours(12)
                + Duration::minutes(40)
                + Duration::seconds(50)
        );
    }
    #[test]
    fn deserialize_durations() {
        assert_eq!("P3D", Duration::days(3).to_string());
        assert_eq!("P1000D", Duration::days(1000).to_string());
        assert_eq!("PT10M", Duration::minutes(10).to_string());
        assert_eq!("P10M", Duration::months(10).to_string());
        assert_eq!(
            "P2M3D",
            (Duration::months(2) + Duration::days(3)).to_string()
        );
        assert_eq!("P1Y", Duration::years(1).to_string());
        assert_eq!(
            "P1Y3D",
            (Duration::years(1) + Duration::days(3)).to_string()
        );
        assert_eq!("PT50S", Duration::seconds(50).to_string());
        assert_eq!("PT40M", Duration::minutes(40).to_string());
        assert_eq!(
            "PT40M50S",
            (Duration::minutes(40) + Duration::seconds(50)).to_string()
        );
        assert_eq!(
            "PT12H40M50S",
            (Duration::hours(12) + Duration::minutes(40) + Duration::seconds(50)).to_string()
        );
        assert_eq!(
            "P1Y2M3DT12H40M50S",
            (Duration::years(1)
                + Duration::months(2)
                + Duration::days(3)
                + Duration::hours(12)
                + Duration::minutes(40)
                + Duration::seconds(50))
            .to_string()
        );
    }

    #[test]
    /// Test parsing a task from a string using different methods
    fn parse_task() {
        let task_str = r#"
        {
            "id": 0,
            "description": "Task to do.",
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
        }
        "#;
        let task = task_str.parse::<Task>().unwrap();
        assert_eq!(task.id, Some(0));

        let task = Task::from(task_str);
        assert_eq!(task.id, Some(0));

        let task = Task::from(task_str.to_string());
        assert_eq!(task.id, Some(0));

        let task: Task = task_str.into();
        assert_eq!(task.id, Some(0));

        let task: Task = task_str.to_string().into();
        assert_eq!(task.id, Some(0));
    }
    #[test]
    fn convert_durations() {
        use std::time;

        let duration = time::Duration::from_secs(50);
        let tasklib_duration: Duration = duration.into();
        assert_eq!(tasklib_duration, Duration::seconds(50));

        let duration = time::Duration::from_secs((40 * 60) + 50);
        let tasklib_duration: Duration = duration.into();
        assert_eq!(tasklib_duration.to_string(), "PT2450S");
        assert_eq!(
            tasklib_duration,
            Duration::minutes(40) + Duration::seconds(50)
        );

        let chrono_duration = chrono::Duration::seconds(50);
        let tasklib_duration: Duration = chrono_duration.into();
        assert_eq!(tasklib_duration, Duration::seconds(50));

        let duration = time::Duration::from_secs(50);
        let tasklib_duration = Duration::from(duration);
        assert_eq!(tasklib_duration, Duration::seconds(50));

        let chrono_duration = chrono::Duration::seconds(50);
        let tasklib_duration = Duration::from(chrono_duration);
        assert_eq!(tasklib_duration, Duration::seconds(50));
    }
    #[test]
    fn test_udas() {
        // Uses elapsed, a duration type UDA
        let task_str = r#"
        {
            "id": 0,
            "description": "Task to do.",
            "end": "20220131T083000Z",
            "entry": "20220131T083000Z",
            "modified": "20220131T083000Z",
            "elapsed": "PT2H",
            "project": "Daily",
            "start": "20220131T083000Z",
            "status": "pending",
            "uuid": "d67fce70-c0b6-43c5-affc-a21e64567d40",
            "tags": [
                "WORK"
            ],
            "urgency": 9.91234
        }
        "#;
        let task = task_str.parse::<Task>().unwrap();
        assert_eq!(task.udas().get("elapsed").unwrap(), "PT2H");
    }

    #[test]
    /// Test the different types of UDA values
    fn test_udas_types() {
        // String
        let task_str = r#"
        {
            "id": 0,
            "description": "Task to do.",
            "end": "20220131T083000Z",
            "entry": "20220131T083000Z",
            "modified": "20220131T083000Z",
            "elapsed": "PT2H",
            "project": "Daily",
            "start": "20220131T083000Z",
            "status": "pending",
            "uuid": "d67fce70-c0b6-43c5-affc-a21e64567d40",
            "tags": [
                "WORK"
            ],
            "urgency": 9.91234
        }
        "#;
        let task = task_str.parse::<Task>().unwrap();
        assert_eq!(task.udas().get("elapsed").unwrap(), "PT2H");

        // Number (integer)
        let task_str = r#"
        {
            "id": 0,
            "description": "Task to do.",
            "end": "20220131T083000Z",
            "entry": "20220131T083000Z",
            "modified": "20220131T083000Z",
            "elapsed": 2,
            "project": "Daily",
            "start": "20220131T083000Z",
            "status": "pending",
            "uuid": "d67fce70-c0b6-43c5-affc-a21e64567d40",
            "tags": [
                "WORK"
            ],
            "urgency": 9.91234
        }
        "#;
        let task = task_str.parse::<Task>().unwrap();
        assert_eq!(task.udas().get("elapsed").unwrap(), &2);

        // Number (float)
        let task_str = r#"
        {
            "id": 0,
            "description": "Task to do.",
            "end": "20220131T083000Z",
            "entry": "20220131T083000Z",
            "modified": "20220131T083000Z",
            "elapsed": 2.5,
            "project": "Daily",
            "start": "20220131T083000Z",
            "status": "pending",
            "uuid": "d67fce70-c0b6-43c5-affc-a21e64567d40",
            "tags": [
                "WORK"
            ],
            "urgency": 9.91234
        }
        "#;
        let task = task_str.parse::<Task>().unwrap();
        assert_eq!(task.udas().get("elapsed").unwrap(), &2.5);
    }

    #[test]
    fn builder() {
        use chrono::ParseError;
        use chrono::TimeZone;

        /// str -> DateTime<Utc>
        ///
        /// e.g."20220131T083000Z" -> DateTime<Utc>
        fn tw_str_to_dt(s: &str) -> Result<DateTime<Utc>, ParseError> {
            Utc.datetime_from_str(s, "%Y%m%dT%H%M%SZ")
        }

        let task = TaskBuilder::new()
            .description("Task to do.")
            .end(tw_str_to_dt("20220131T083000Z").unwrap())
            .entry(tw_str_to_dt("20220131T083000Z").unwrap())
            .modified(tw_str_to_dt("20220131T083000Z").unwrap())
            .project("Daily")
            .start(tw_str_to_dt("20220131T083000Z").unwrap())
            .status(Status::Pending)
            .uuid("d67fce70-c0b6-43c5-affc-a21e64567d40")
            .tags(vec!["WORK"])
            .urgency(9.91234)
            .parent("d67fce70-c0b6-43c5-affc-a21e64567d40")
            .build();
        assert_eq!(task.id(), &None);
    }

    #[test]
    fn deserialize_task() {
        // Task should not include null or empty fields when deserialized to JSON
        let task_str = r#"
        {
            "uuid": "d67fce70-c0b6-43c5-affc-a21e64567d40",
            "description": "Task to do.",
            "status": "pending",
            "entry": "20220131T083000Z",
            "modified": "20220131T083000Z"
        }
        "#;
        let task = task_str.parse::<Task>().unwrap();
        let task_json = serde_json::to_string(&task).unwrap();
        let expected_task_json = r#"{"uuid":"d67fce70-c0b6-43c5-affc-a21e64567d40","description":"Task to do.","entry":"20220131T083000Z","modified":"20220131T083000Z","status":"pending"}"#;
        assert_eq!(task_json, expected_task_json);
    }
}

pub mod prelude {
    pub use super::Task;
    pub use super::TaskBuilder;
    pub use crate::duration::Duration;
    pub use crate::udas::UdaValue;
}
