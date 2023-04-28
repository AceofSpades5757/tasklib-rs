//! Minimal example using `tasklib`.
//!
//! ```rust
//! use tasklib::Task;
//!
//! let json = r#"
//! {
//!   "id": 0,
//!   "description": "Task to do.",
//!   "elapsed": "PT2H",
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
//! let task: Task = serde_json::from_str(json).expect("valid json parsed into a task");
//! // Getting a String from your Serialized Task
//! let task_str: String = serde_json::to_string(&task).expect("valid json string representing a task");
//! ```

use std::fmt;
use std::str::FromStr;

use chrono::{offset::Utc, DateTime, NaiveDateTime};
use regex::Regex;
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

const FORMAT: &str = "%Y%m%dT%H%M%SZ";

fn tw_str_to_dt<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
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
                NaiveDateTime::parse_from_str(v, FORMAT).expect("string turned into datetime"),
                Utc,
            ))
        }
    }
    deserializer.deserialize_any(DateTimeStringVisitor)
}

fn tw_str_to_dt_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
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
                NaiveDateTime::parse_from_str(v, FORMAT).expect("string turned into datetime"),
                Utc,
            )))
        }
    }
    deserializer.deserialize_any(DateTimeStringVisitor)
}

/// DateTime<Utc> -> String
fn tw_dt_to_str<S: Serializer>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&dt.format(&FORMAT).to_string())
}

/// DateTime<Utc> -> String
fn tw_dt_to_str_opt<S: Serializer>(dt: &Option<DateTime<Utc>>, s: S) -> Result<S::Ok, S::Error> {
    match dt {
        Some(dt) => s.serialize_str(&dt.format(&FORMAT).to_string()),
        None => s.serialize_str(""),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    id: usize,
    uuid: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    elapsed: Option<Duration>,
    #[serde(
        serialize_with = "tw_dt_to_str_opt",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "tw_str_to_dt_opt",
        default
    )]
    start: Option<DateTime<Utc>>,
    #[serde(
        serialize_with = "tw_dt_to_str_opt",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "tw_str_to_dt_opt",
        default
    )]
    end: Option<DateTime<Utc>>,
    #[serde(serialize_with = "tw_dt_to_str", deserialize_with = "tw_str_to_dt")]
    entry: DateTime<Utc>,
    #[serde(serialize_with = "tw_dt_to_str", deserialize_with = "tw_str_to_dt")]
    modified: DateTime<Utc>,
    #[serde(default)]
    project: String,
    status: Status,
    #[serde(default)]
    tags: Vec<String>,
    urgency: f64,
    #[serde(default)]
    annotations: Vec<Annotation>,
}

/// Getters (Immutable)
impl Task {
    pub fn id(&self) -> &usize {
        &self.id
    }
    pub fn uuid(&self) -> &str {
        &self.uuid
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn elapsed(&self) -> Option<&Duration> {
        self.elapsed.as_ref()
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
}

impl FromStr for Task {
    //type Err = ParseError;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Annotation {
    #[serde(serialize_with = "tw_dt_to_str", deserialize_with = "tw_str_to_dt")]
    entry: DateTime<Utc>,
    description: String,
}

// #[derive(Debug, Serialize)]
#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Default)]
pub struct Duration {
    years: u32,
    months: u32,
    days: u32,
    hours: u32,
    minutes: u32,
    seconds: u32,
}

impl Duration {
    fn to_string(&self) -> String {
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
        }
        "#;
        let task = task_str.parse::<Task>().unwrap();
        assert_eq!(task.id, 0);

        let task = Task::from(task_str);
        assert_eq!(task.id, 0);

        let task = Task::from(task_str.to_string());
        assert_eq!(task.id, 0);
    }
}
