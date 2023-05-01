//! Minimal example using `tasklib`.
//!
//! ```rust
//! use tasklib::Task;
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

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::time;
use uuid::Uuid;

use chrono::{self, offset::Utc, DateTime, NaiveDateTime};
use regex::Regex;
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use udas::Uda;

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
                NaiveDateTime::parse_from_str(v, DATETIME_FORMAT).expect("string turned into datetime"),
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
                NaiveDateTime::parse_from_str(v, DATETIME_FORMAT).expect("string turned into datetime"),
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

/// Taskwarrior UDA to String serializer
///
/// Uda -> String
fn tw_uda_to_str<S: Serializer>(uda: Uda, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&uda.to_string())
}

/// Taskwarrior String to UDA deserializer
///
/// String -> Uda
fn tw_str_to_uda<'de, D>(deserializer: D) -> Result<Uda, D::Error>
where
    D: Deserializer<'de>,
{
    struct UdaStringVisitor;

    impl<'de> de::Visitor<'de> for UdaStringVisitor {
        type Value = Uda;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containg UDA data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            //Ok(Uda::from(v).expect("string turned into UDA"))
            Ok(Uda::from(v))
        }
    }
    deserializer.deserialize_any(UdaStringVisitor)
}

/// See all columns using `task columns` and `task _columns`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    id: usize,
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
    #[serde(serialize_with = "tw_dt_to_str_se", deserialize_with = "tw_str_to_dt_de")]
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
    depends: Vec<Uuid>,
    #[serde(default)]
    /// <https://taskwarrior.org/docs/commands/columns/>
    /// Type: numeric
    imask: Option<i64>,
    #[serde(default)]
    mask: Option<String>,
    #[serde(default)]
    parent: Option<Uuid>,
    /// Used with recurance templates.
    #[serde(default)]
    recur: Option<Duration>,
    #[serde(serialize_with = "tw_dt_to_str_se", deserialize_with = "tw_str_to_dt_de")]
    modified: DateTime<Utc>,
    #[serde(default)]
    project: String,
    status: Status,
    #[serde(default)]
    tags: Vec<String>,
    urgency: f64,
    #[serde(default)]
    annotations: Vec<Annotation>,
    //#[serde(default)]
    //udas: HashMap<String, Uda>,
    // Any other attributes are UDAs and should be stored in a HashMap as a string type UDA.
    //#[serde(flatten)]
    //#[serde(flatten, serialize_with = "tw_uda_to_str", deserialize_with = "tw_str_to_uda")]
    //#[serde(serialize_with = "tw_uda_to_str", deserialize_with = "tw_str_to_uda")]
    //udas: HashMap<String, Uda>,
}

/// Getters (Immutable)
impl Task {
    pub fn id(&self) -> &usize {
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
    pub fn urgency(&self) -> &f64 {
        &self.urgency
    }
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
    pub fn udas(&self) -> &HashMap<String, Uda> {
        &self.udas
    }
}

/*
 * TODO: Implement from_stream (takes a json stream and returns a task)
 * TODO: Implement from_stdin (automatically reads from stdin and returns a task)

/// Constructors
impl Task {
    fn from_stream(stream: &mut impl Read) -> Result<Self, Error> {
        let mut buf = String::new();
        stream.read_to_string(&mut buf)?;
        let task: Task = serde_json::from_str(&buf)?;
        Ok(task)
    }
    fn from_stdin() -> Result<Self, Error> {
        let mut stdin = io::stdin();
        Task::from_stream(&mut stdin)
    }
}
*/

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
    #[serde(serialize_with = "tw_dt_to_str_se", deserialize_with = "tw_str_to_dt_de")]
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

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Duration {
    years: u32,
    months: u32,
    days: u32,
    hours: u32,
    minutes: u32,
    seconds: u32,
}

impl Duration {
    pub fn to_string(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str("P");
        if self.years > 0 {
            buffer.push_str(&format!("{}Y", self.years))
        }
        if self.months > 0 {
            buffer.push_str(&format!("{}M", self.months))
        }
        if self.days > 0 {
            buffer.push_str(&format!("{}D", self.days))
        }
        if self.hours > 0 || self.minutes > 0 || self.seconds > 0 {
            buffer.push_str("T")
        }
        if self.hours > 0 {
            buffer.push_str(&format!("{}H", self.hours))
        }
        if self.minutes > 0 {
            buffer.push_str(&format!("{}M", self.minutes))
        }
        if self.seconds > 0 {
            buffer.push_str(&format!("{}S", self.seconds))
        }
        buffer
    }
}

impl Duration {
    /// Shift the values around to their reasonable maxes.
    ///
    /// e.g. seconds should never be more then 59
    fn smooth(&mut self) {
        if self.seconds > 59 {
            self.minutes += self.seconds / 60;
            self.seconds = self.seconds % 60;
        }
        if self.minutes > 59 {
            self.hours += self.minutes / 60;
            self.minutes = self.minutes % 60;
        }
        if self.hours > 23 {
            self.days += self.hours / 24;
            self.hours = self.hours % 24;
        }
        if self.days > 30 {
            self.months += self.days / 30;
            self.days = self.days % 30;
        }
        if self.months > 11 {
            self.years += self.months / 12;
            self.months = self.months % 12;
        }
    }
}

impl From<String> for Duration {
    fn from(s: String) -> Self {
        Duration::from_str(&s).expect("string turned into duration")
    }
}

impl From<&str> for Duration {
    fn from(s: &str) -> Self {
        Duration::from_str(s).expect("string turned into duration")
    }
}

impl From<time::Duration> for Duration {
    fn from(duration: time::Duration) -> Self {
        let mut dur = Duration {
            seconds: duration.as_secs() as u32,
            ..Default::default()
        };
        dur.smooth();
        dur
    }
}

impl From<chrono::Duration> for Duration {
    fn from(duration: chrono::Duration) -> Self {
        let mut dur = Duration {
            seconds: duration.num_seconds() as u32,
            ..Default::default()
        };
        dur.smooth();
        dur
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Color", 3)?;
        state.serialize_field("r", &self.hours)?;
        state.serialize_field("g", &self.minutes)?;
        state.serialize_field("b", &self.seconds)?;
        state.end()
    }
}

impl FromStr for Duration {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(concat!(
            "P",
            r"((?P<years>\d+)Y)?",
            r"((?P<months>\d+)M)?",
            r"((?P<days>\d+)D)?",
            r"(T",
            r"((?P<hours>\d+)H)?",
            r"((?P<minutes>\d+)M)?",
            r"((?P<seconds>\d+)S)?)?",
        ))
        .expect("valid regex");
        let captures = re.captures(s).expect("valid duration string for capture");

        let years = if let Some(years) = captures.name("years") {
            years
                .as_str()
                .parse::<u32>()
                .expect("valid number as string")
        } else {
            0
        };
        let months = if let Some(months) = captures.name("months") {
            months
                .as_str()
                .parse::<u32>()
                .expect("valid number as string")
        } else {
            0
        };
        let days = if let Some(days) = captures.name("days") {
            days.as_str()
                .parse::<u32>()
                .expect("valid number as string")
        } else {
            0
        };
        let hours = if let Some(hours) = captures.name("hours") {
            hours
                .as_str()
                .parse::<u32>()
                .expect("valid number as string")
        } else {
            0
        };
        let minutes = if let Some(minutes) = captures.name("minutes") {
            minutes
                .as_str()
                .parse::<u32>()
                .expect("valid number as string")
        } else {
            0
        };
        let seconds = if let Some(seconds) = captures.name("seconds") {
            seconds
                .as_str()
                .parse::<u32>()
                .expect("valid number as string")
        } else {
            0
        };

        Ok(Duration {
            years,
            months,
            days,
            hours,
            minutes,
            seconds,
        })
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

/// TODO: Add udas.
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
    //udas: Option<HashMap<String, Uda>>,
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
    //pub fn uda(mut self, uda: Uda) -> Self {
        //self.uda = Some(uda);
        //self
    //}
}

impl TaskBuilder {
    pub fn new() -> Self {
        TaskBuilder {
            ..Default::default()
        }
    }
    pub fn build(self) -> Task {
        Task {
            id: self.id.unwrap(),
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
            urgency: self.urgency.unwrap_or(0.0),
            /// FIXME
            udas: HashMap::new(),
        }
    }
}

mod udas {

    use super::Duration;
    use super::tw_dt_to_str_opt_se;
    use super::tw_str_to_dt_opt_de;
    use super::tw_dt_to_str_se;
    use super::tw_str_to_dt_de;
    use chrono::{self, offset::Utc, DateTime};

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
    #[derive(Debug, Clone, PartialEq)]
    pub enum Uda {
        String{
            label: String,
            name: String,
            value: String,
            default: String,
            values: Vec<String>,
            coefficient: Option<f32>,
        },
        Numeric{
            name: String,
            label: String,
            value: f64,
            default: f64,
            coefficient: Option<f32>,
        },
        Date{
            name: String,
            label: String,
            #[serde(serialize_with = "tw_dt_to_str_se", deserialize_with = "tw_str_to_dt_de")]
            value: DateTime<Utc>,
            #[serde(
                serialize_with = "tw_dt_to_str_opt_se",
                skip_serializing_if = "Option::is_none",
                deserialize_with = "tw_str_to_dt_opt_de",
                default
            )]
            default: Option<DateTime<Utc>>,
            coefficient: Option<f32>,
        },
        Duration{
            name: String,
            label: String,
            value: Duration,
            default: Option<Duration>,
            coefficient: Option<f32>,
        },
    }

    impl Uda {
        pub fn to_string(&self) -> String {
            match self {
                Uda::String { name, value, .. } => format!("{}", value),
                Uda::Numeric { name, value, .. } => format!("{}", value),
                Uda::Date { name, value, .. } => format!("{}", value),
                Uda::Duration { name, value, .. } => format!("{}", value.to_string()),
            }
        }
    }

    impl From<Uda> for String {
        fn from(uda: Uda) -> Self {
            uda.to_string()
        }
    }
    
    /// FIXME
    impl From<String> for Uda {
        fn from(s: String) -> Self {
            Uda::String {
                name: s.clone(),
                label: s.clone(),
                value: s.clone(),
                default: s.clone(),
                values: vec![s.clone()],
                coefficient: None,
            }
        }
    }

    impl From<&str> for Uda {
        fn from(s: &str) -> Self {
            Uda::String {
                name: s.to_string(),
                label: s.to_string(),
                value: s.to_string(),
                default: s.to_string(),
                values: vec![s.to_string()],
                coefficient: None,
            }
        }
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
                Uda::String{value, ..} => value == other,
                _ => false,
            }
        }
    }

    impl PartialEq<str> for Uda {
        fn eq(&self, other: &str) -> bool {
            match self {
                Uda::String{value, ..} => value == other,
                _ => false,
            }
        }
    }

    impl Uda {
        /// Get the type of the UDA as a string
        pub fn r#type(&self) -> String {
            match self {
                Uda::String{..} => "string".to_string(),
                Uda::Numeric{..} => "numeric".to_string(),
                Uda::Date{..} => "date".to_string(),
                Uda::Duration{..} => "duration".to_string(),
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
        fn test() {
            let uda_1 = Uda::String {
                label: "my_uda".to_string(),
                value: "my_value".to_string(),
                default: "my_default".to_string(),
                values: vec!["my_value".to_string(), "my_value_2".to_string()],
                coefficient: Some(1.0),
            };
            let uda_2 = Uda::String {
                label: "my_uda".to_string(),
                value: "my_value".to_string(),
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
    fn serialize_durations() {
        assert_eq!(
            "P3D".parse::<Duration>().unwrap(),
            Duration {
                days: 3,
                ..Default::default()
            }
        );
        assert_eq!(
            "P1000D".parse::<Duration>().unwrap(),
            Duration {
                days: 1000,
                ..Default::default()
            }
        );
        assert_eq!(
            "PT10M".parse::<Duration>().unwrap(),
            Duration {
                minutes: 10,
                ..Default::default()
            }
        );
        assert_eq!(
            "P10M".parse::<Duration>().unwrap(),
            Duration {
                months: 10,
                ..Default::default()
            }
        );
        assert_eq!(
            "P2M3D".parse::<Duration>().unwrap(),
            Duration {
                months: 2,
                days: 3,
                ..Default::default()
            }
        );
        assert_eq!(
            "P1Y".parse::<Duration>().unwrap(),
            Duration {
                years: 1,
                ..Default::default()
            }
        );
        assert_eq!(
            "P1Y3D".parse::<Duration>().unwrap(),
            Duration {
                years: 1,
                days: 3,
                ..Default::default()
            }
        );
        assert_eq!(
            "PT50S".parse::<Duration>().unwrap(),
            Duration {
                seconds: 50,
                ..Default::default()
            }
        );
        assert_eq!(
            "PT40M".parse::<Duration>().unwrap(),
            Duration {
                minutes: 40,
                ..Default::default()
            }
        );
        assert_eq!(
            "PT40M50S".parse::<Duration>().unwrap(),
            Duration {
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
        );
        assert_eq!(
            "PT12H40M50S".parse::<Duration>().unwrap(),
            Duration {
                hours: 12,
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
        );
        assert_eq!(
            "P1Y2M3DT12H40M50S".parse::<Duration>().unwrap(),
            Duration {
                years: 1,
                months: 2,
                days: 3,
                hours: 12,
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
        );
        assert_eq!(
            {
                let dur: Duration = "P1Y2M3DT12H40M50S".into();
                dur
            },
            Duration {
                years: 1,
                months: 2,
                days: 3,
                hours: 12,
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
        );
        assert_eq!(
            {
                let dur: Duration = "P1Y2M3DT12H40M50S".to_string().into();
                dur
            },
            Duration {
                years: 1,
                months: 2,
                days: 3,
                hours: 12,
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
        );
        assert_eq!(
            {
                let dur: Duration = Duration::from("P1Y2M3DT12H40M50S");
                dur
            },
            Duration {
                years: 1,
                months: 2,
                days: 3,
                hours: 12,
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
        );
        assert_eq!(
            {
                let dur: Duration = Duration::from("P1Y2M3DT12H40M50S".to_string());
                dur
            },
            Duration {
                years: 1,
                months: 2,
                days: 3,
                hours: 12,
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
        );
    }
    #[test]
    fn deserialize_durations() {
        assert_eq!(
            "P3D",
            Duration {
                days: 3,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "P1000D",
            Duration {
                days: 1000,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "PT10M",
            Duration {
                minutes: 10,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "P10M",
            Duration {
                months: 10,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "P2M3D",
            Duration {
                months: 2,
                days: 3,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "P1Y",
            Duration {
                years: 1,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "P1Y3D",
            Duration {
                years: 1,
                days: 3,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "PT50S",
            Duration {
                seconds: 50,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "PT40M",
            Duration {
                minutes: 40,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "PT40M50S",
            Duration {
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "PT12H40M50S",
            Duration {
                hours: 12,
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
            .to_string()
        );
        assert_eq!(
            "P1Y2M3DT12H40M50S",
            Duration {
                years: 1,
                months: 2,
                days: 3,
                hours: 12,
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
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
        assert_eq!(task.id, 0);

        let task = Task::from(task_str);
        assert_eq!(task.id, 0);

        let task = Task::from(task_str.to_string());
        assert_eq!(task.id, 0);

        let task: Task = task_str.into();
        assert_eq!(task.id, 0);

        let task: Task = task_str.to_string().into();
        assert_eq!(task.id, 0);
    }
    #[test]
    fn convert_durations() {

        let duration = time::Duration::from_secs(50);
        let tasklib_duration: Duration = duration.into();
        assert_eq!(
            tasklib_duration,
            Duration {
                seconds: 50,
                ..Default::default()
            }
        );

        let duration = time::Duration::from_secs((40 * 60) + 50);
        let tasklib_duration: Duration = duration.into();
        assert_eq!(tasklib_duration.to_string(), "PT40M50S");
        assert_eq!(
            tasklib_duration,
            Duration {
                minutes: 40,
                seconds: 50,
                ..Default::default()
            }
        );

        let chrono_duration = chrono::Duration::seconds(50);
        let tasklib_duration: Duration = chrono_duration.into();
        assert_eq!(
            tasklib_duration,
            Duration {
                seconds: 50,
                ..Default::default()
            }
        );

        let duration = time::Duration::from_secs(50);
        let tasklib_duration = Duration::from(duration);
        assert_eq!(
            tasklib_duration,
            Duration {
                seconds: 50,
                ..Default::default()
            }
        );

        let chrono_duration = chrono::Duration::seconds(50);
        let tasklib_duration = Duration::from(chrono_duration);
        assert_eq!(
            tasklib_duration,
            Duration {
                seconds: 50,
                ..Default::default()
            }
        );
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
        //assert_eq!(task.udas().get("elapsed").unwrap(), Duration::from("PT2H"));
        // Default type is string
        assert_eq!(task.udas().get("elapsed").unwrap(), "PT2H");
    }

    #[test]
    fn test_builder() {

        use chrono::TimeZone;
        use chrono::ParseError;

        /// str -> DateTime<Utc>
        ///
        /// e.g."20220131T083000Z" -> DateTime<Utc>
        fn tw_str_to_dt(s: &str) -> Result<DateTime<Utc>, ParseError> {
            Utc.datetime_from_str(s, "%Y%m%dT%H%M%SZ")
        }

        let task = TaskBuilder::new()
            .id(0)
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
        assert_eq!(task.id, 0);
    }
}

pub mod prelude {
    pub use super::Task;
    pub use super::TaskBuilder;
}
