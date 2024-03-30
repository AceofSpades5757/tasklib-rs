//! FIXME: 1 month + 1 month should really be 2 months and not converted to 60 days.
//!   * Should this really be fixed? `task calc` will calculate `1m + 1m` as 60 days.
use crate::UdaValue;
use std::convert::TryFrom;
use std::fmt;
use std::ops;
use std::str::FromStr;
use std::time;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::character::complete::space0;
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::error::context;
use nom::sequence::tuple;
use nom::IResult;
use serde::{Deserialize, Serialize};

/// Special duration types.
///
/// * weekdays
#[derive(Debug, Default, Clone)]
enum Special {
    /// Represented by "weekdays" in Taskwarrior's `recur` attribute.
    Weekdays,
    #[default]
    None,
}

#[derive(Debug, Default, Clone)]
pub struct Duration {
    years: u32,
    months: u32,
    days: u32,
    hours: u32,
    minutes: u32,
    seconds: u32,
    /// Special circumstances in Taskwarrior, such as "weekdays" that needs to be specially
    /// formatted during serialization and cannot be represented using duration alone.
    special: Special,
    /// If deserialized, this will be the source.
    ///
    /// Used to avoid changes in the serialized output.
    ///
    /// e.g. P1M is equivalent to P30D for a Taskwarrior duration, but this is not equivalent when
    /// used in recurring tasks. `tasklib` will properly parse the P1M but it won't turn 30 days
    /// into a month (to avoid data loss).
    source: Option<String>,
}

/// Constructors
impl Duration {
    pub fn seconds(seconds: u32) -> Self {
        Duration {
            seconds,
            ..Default::default()
        }
    }
    pub fn days(days: u32) -> Self {
        Duration {
            days,
            ..Default::default()
        }
    }
    pub fn hours(hours: u32) -> Self {
        Duration {
            hours,
            ..Default::default()
        }
    }
    pub fn minutes(minutes: u32) -> Self {
        Duration {
            minutes,
            ..Default::default()
        }
    }
    pub fn weeks(weeks: u32) -> Self {
        Duration {
            days: weeks * 7,
            ..Default::default()
        }
    }
    pub fn months(months: u32) -> Self {
        Duration {
            months,
            ..Default::default()
        }
    }
    pub fn years(years: u32) -> Self {
        Duration {
            years,
            ..Default::default()
        }
    }
}

/// Conversion Methods
impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Source
        //
        // Return the same output if an input was originally given.
        if let Some(ref source) = self.source {
            return write!(f, "{}", *source);
        }
        // Special Circumstances
        //
        // e.g. "weekdays"
        if let Special::Weekdays = self.special {
            return write!(f, "weekdays");
        }

        let mut buffer = String::new();
        buffer.push('P');
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
            buffer.push('T')
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
        write!(f, "{buffer}")
    }
}

impl Duration {
    pub fn num_seconds(&self) -> u32 {
        let seconds_per_minute = 60;
        let seconds_per_hour = 60 * seconds_per_minute;
        let seconds_per_day = 24 * seconds_per_hour;
        let seconds_per_month = 30 * seconds_per_day;
        let seconds_per_year = 365 * seconds_per_day;

        self.seconds
            + self.minutes * seconds_per_minute
            + self.hours * seconds_per_hour
            + self.days * seconds_per_day
            + self.months * seconds_per_month
            + self.years * seconds_per_year
    }
}

impl Duration {
    /// Smooth values
    ///
    /// Will not convert months and days as months may have a different number of days.
    ///
    /// e.g. PT7200S -> PT2H
    pub fn smooth(&mut self) {
        self.minutes += self.seconds / 60;
        self.seconds %= 60;

        self.hours += self.minutes / 60;
        self.minutes %= 60;

        self.days += self.hours / 24;
        self.days %= 24;

        self.years += self.months / 12;
        self.months %= 12;
    }
}

impl ops::Add for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Duration {
            years: self.years + other.years,
            months: self.months + other.months,
            days: self.days + other.days,
            hours: self.hours + other.hours,
            minutes: self.minutes + other.minutes,
            seconds: self.seconds + other.seconds,
            ..Default::default()
        }
    }
}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        self.num_seconds() == other.num_seconds()
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

impl FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = s.to_string();
        let (_, mut duration) = parse_duration(s).map_err(|e| format!("{e}"))?;
        duration.source = Some(source);
        Ok(duration)
    }
}

impl From<Duration> for String {
    fn from(duration: Duration) -> Self {
        duration.to_string()
    }
}

impl From<time::Duration> for Duration {
    fn from(duration: time::Duration) -> Self {
        // FIXME: Smooth this
        Duration {
            seconds: duration.as_secs() as u32,
            ..Default::default()
        }
    }
}

/// FIXME: Add proper error return type
impl TryFrom<UdaValue> for Duration {
    //type Error = Box<dyn Error>;
    type Error = ();
    fn try_from(uda_value: UdaValue) -> Result<Self, Self::Error> {
        match uda_value {
            UdaValue::String(s) => match s.parse::<Duration>() {
                Ok(d) => Ok(d),
                Err(_) => Err(()),
            },
            UdaValue::Duration(d) => Ok(d),
            // All other types are not supported
            _ => Err(()),
        }
    }
}

impl From<chrono::Duration> for Duration {
    fn from(duration: chrono::Duration) -> Self {
        Duration {
            seconds: duration.num_seconds() as u32,
            ..Default::default()
        }
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Duration::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Parse seconds with a number
fn parse_seconds_ordinal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("seconds", |input: &'a str| {
        // Digit
        let (input, seconds) = digit1(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Seconds literal
        let (input, _) = alt((
            tag("seconds"),
            tag("second"),
            tag("secs"),
            tag("sec"),
            tag("s"),
        ))(input)?;
        // Turn into a duration
        Ok((input, Duration::seconds(seconds.parse::<u32>().unwrap())))
    })(input)
}

/// Parse seconds without a number
///
/// * `second`
/// * `sec`
fn parse_seconds_literal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("seconds", |input: &'a str| {
        // Seconds literal
        let (input, _) = alt((tag("second"), tag("sec")))(input)?;
        // Turn into a duration
        Ok((input, Duration::seconds(1)))
    })(input)
}

/// Parse seconds with or without a number
///
/// e.g. `5 seconds`, `second`, `sec`
fn parse_seconds<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("seconds", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Parse using any of the known formats
        let (input, duration) = alt((parse_seconds_ordinal, parse_seconds_literal))(input)?;
        Ok((input, duration))
    })(input)
}

/// Parse minutes with a number
fn parse_minutes_ordinal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("minutes", |input: &'a str| {
        // Digit
        let (input, minutes) = digit1(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Minutes literal
        let (input, _) = alt((tag("minutes"), tag("minute"), tag("mins"), tag("min")))(input)?;
        // Turn into a duration
        Ok((input, Duration::minutes(minutes.parse::<u32>().unwrap())))
    })(input)
}

/// Parse minutes without a number
///
/// * `minute`
/// * `min`
fn parse_minutes_literal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("minutes", |input: &'a str| {
        // Minutes literal
        let (input, _) = alt((tag("minute"), tag("min")))(input)?;
        // Turn into a duration
        Ok((input, Duration::minutes(1)))
    })(input)
}

/// Parse minutes with or without a number
///
/// e.g. `5 minutes`, `minute`, `min`
fn parse_minutes<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("minutes", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Parse using any of the known formats
        let (input, duration) = alt((parse_minutes_ordinal, parse_minutes_literal))(input)?;
        Ok((input, duration))
    })(input)
}

/// Parse hours with a number
fn parse_hours_ordinal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("hours", |input: &'a str| {
        // Digit
        let (input, hours) = digit1(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Hours literal
        let (input, _) = alt((tag("hours"), tag("hour"), tag("hrs"), tag("hr"), tag("h")))(input)?;
        // Turn into a duration
        Ok((input, Duration::hours(hours.parse::<u32>().unwrap())))
    })(input)
}

/// Parse hours without a number
///
/// * `hour`
/// * `hr`
fn parse_hours_literal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("hours", |input: &'a str| {
        // Hours literal
        let (input, _) = alt((tag("hour"), tag("hr")))(input)?;
        // Turn into a duration
        Ok((input, Duration::hours(1)))
    })(input)
}

/// Parse hours with or without a number
///
/// e.g. `5 hours`, `hour`, `hr`
fn parse_hours<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("hours", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Parse using any of the known formats
        let (input, duration) = alt((parse_hours_ordinal, parse_hours_literal))(input)?;
        Ok((input, duration))
    })(input)
}

/// Parse days with a number
fn parse_days_ordinal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("days", |input: &'a str| {
        // Digit
        let (input, days) = digit1(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Days literal
        let (input, _) = alt((tag("days"), tag("day"), tag("daily"), tag("d")))(input)?;
        // Turn into a duration
        Ok((input, Duration::days(days.parse::<u32>().unwrap())))
    })(input)
}

/// Parse days without a number
///
/// * `daily`
/// * `day`
fn parse_days_literal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("days", |input: &'a str| {
        // Days literal
        let (input, _) = alt((tag("daily"), tag("day")))(input)?;
        // Turn into a duration
        Ok((input, Duration::days(1)))
    })(input)
}

/// Parse days with or without a number
///
/// e.g. `5 days`, `day`, `daily`
fn parse_days<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("days", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Parse using any of the known formats
        let (input, duration) = alt((parse_days_ordinal, parse_days_literal))(input)?;
        Ok((input, duration))
    })(input)
}

/// Parse weeks with a number
fn parse_weeks_ordinal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("weeks", |input: &'a str| {
        // Digit
        let (input, weeks) = digit1(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Weeks literal
        let (input, _) = alt((
            tag("weeks"),
            tag("weekly"),
            tag("week"),
            tag("wks"),
            tag("wk"),
            tag("w"),
        ))(input)?;
        // Turn into a duration
        Ok((input, Duration::weeks(weeks.parse::<u32>().unwrap())))
    })(input)
}

/// Parse weeks without a number
///
/// * `weekly`
/// * `week`
/// * `wk`
fn parse_weeks_literal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("weeks", |input: &'a str| {
        // Weeks literal
        let (input, _) = alt((tag("weekly"), tag("week"), tag("wk")))(input)?;
        // Turn into a duration
        Ok((input, Duration::weeks(1)))
    })(input)
}

/// Parse weeks with or without a number
///
/// e.g. `5 weeks`, `week`, `weekly`, `wk`
fn parse_weeks<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("weeks", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Parse using any of the known formats
        let (input, duration) = alt((parse_weeks_ordinal, parse_weeks_literal))(input)?;
        Ok((input, duration))
    })(input)
}

/// Parse months with a number
fn parse_months_ordinal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("months", |input: &'a str| {
        // Digit
        let (input, months) = digit1(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Months literal
        let (input, _) = alt((
            tag("months"),
            tag("monthly"),
            tag("month"),
            tag("mo"),
            tag("m"),
        ))(input)?;
        // Turn into a duration
        Ok((input, Duration::days(30 * months.parse::<u32>().unwrap())))
    })(input)
}

/// Parse months without a number
/// * `monthly`
/// * `month`
/// * `mth`
/// * `mo`
fn parse_months_literal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("months", |input: &'a str| {
        // Months literal
        let (input, _) = alt((tag("monthly"), tag("month"), tag("mth"), tag("mo")))(input)?;
        // Turn into a duration
        Ok((input, Duration::days(30)))
    })(input)
}

/// Parse months with or without a number
///
/// e.g. `5 months`, `month`, `monthly`, `mth`, `mo`
/// Note: months are assumed to be 30 days
fn parse_months<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("months", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Parse using any of the known formats
        let (input, duration) = alt((parse_months_ordinal, parse_months_literal))(input)?;
        Ok((input, duration))
    })(input)
}

/// Parse years with a number
fn parse_years_ordinal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("years", |input: &'a str| {
        // Digit
        let (input, years) = digit1(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Years literal
        let (input, _) = alt((
            tag("years"),
            tag("yearly"),
            tag("year"),
            tag("yrs"),
            tag("yr"),
            tag("y"),
        ))(input)?;
        // Turn into a duration
        Ok((input, Duration::days(365 * years.parse::<u32>().unwrap())))
    })(input)
}

/// Parse years without a number
/// * `yearly`
/// * `year`
/// * `yr`
fn parse_years_literal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("years", |input: &'a str| {
        // Years literal
        let (input, _) = alt((tag("yearly"), tag("year"), tag("yr")))(input)?;
        // Turn into a duration
        Ok((input, Duration::days(365)))
    })(input)
}

/// Parse years with or without a number
/// e.g. `5 years`, `year`, `yearly`, `yr`
/// Note: years are assumed to be 365 days
fn parse_years<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("years", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Parse using any of the known formats
        let (input, duration) = alt((parse_years_ordinal, parse_years_literal))(input)?;
        Ok((input, duration))
    })(input)
}

/// Parse weekdays
///
/// Every weekday, monday through friday
fn parse_weekdays<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("weekdays", |input: &'a str| {
        let source = input.to_string();

        // Any amount of space
        let (input, _) = space0(input)?;
        // Optional ordinal
        let (input, digit) = opt(digit1)(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Weekdays literal
        let (input, _) = alt((tag("weekdays"),))(input)?;

        // If no ordinal, then special should be Special::Weekdays
        let special: Special = if let None = digit {
            Special::Weekdays
        } else {
            Special::None
        };

        let mut duration = Duration::days(digit.unwrap_or("1").parse::<u32>().unwrap());
        duration.special = special;
        duration.source = Some(source);

        // Turn into a duration
        Ok((
            input,
            duration
        ))
    })(input)
}

/// Parse fortnights
/// * `fortnight`
/// * `2 fortnightly`
fn parse_fortnights<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("fortnights", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Optional ordinal
        let (input, digit) = opt(digit1)(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Fortnights literal
        let (input, _) = tag("fortnight")(input)?;
        // Turn into a duration
        Ok((
            input,
            Duration::days(14 * digit.unwrap_or("1").parse::<u32>().unwrap()),
        ))
    })(input)
}

/// Parse sennights
/// * `sennight`
/// * `2 sennight`
///
/// WARNING: Taskwarrior's calc command does not properly handle sennights
fn parse_sennights<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("sennights", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Optional ordinal
        let (input, digit) = opt(digit1)(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Sennights literal
        let (input, _) = tag("sennight")(input)?;
        // Turn into a duration
        Ok((
            input,
            Duration::days(7 * digit.unwrap_or("1").parse::<u32>().unwrap()),
        ))
    })(input)
}

/// Parse biweekly
/// * `biweekly`
///
/// Note: Biweekly is assumed to be 14 days
fn parse_biweekly<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("biweekly", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Optional ordinal
        let (input, digit) = opt(digit1)(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Sennights literal
        let (input, _) = tag("biweekly")(input)?;
        // Turn into a duration
        Ok((
            input,
            Duration::days(14 * digit.unwrap_or("1").parse::<u32>().unwrap()),
        ))
    })(input)
}

/// Parse bimonhtly
/// * `bimonthly`
///
/// Note: Bimonthly is assumed to be 61 days
fn parse_bimonthly<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("bimonthly", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Optional ordinal
        let (input, digit) = opt(digit1)(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Sennights literal
        let (input, _) = tag("bimonthly")(input)?;
        // Turn into a duration
        Ok((
            input,
            Duration::days(61 * digit.unwrap_or("1").parse::<u32>().unwrap()),
        ))
    })(input)
}

/// Parse quarters with a number
///
/// Note: Quarters are assumed to be 91 days
fn parse_quarterly_ordinal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("quarters", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Optional ordinal
        let (input, digit) = digit1(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Quarters literal
        let (input, _) = alt((
            tag("quarterly"),
            tag("quarters"),
            tag("quarter"),
            tag("qrtrs"),
            tag("qrtr"),
            tag("qtr"),
            tag("q"),
        ))(input)?;
        // Turn into a duration
        Ok((input, Duration::days(91 * digit.parse::<u32>().unwrap())))
    })(input)
}

/// Parse quarters without a number
/// * `quarterly`
/// * `quarter`
/// * `qtr`
fn parse_quarterly_literal<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("quarters", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Quarters literal
        let (input, _) = alt((tag("quarterly"), tag("quarter"), tag("qrtr"), tag("qtr")))(input)?;
        // Turn into a duration
        Ok((input, Duration::days(91)))
    })(input)
}

/// Parse quarters
///
/// e.g. `1 quarter`, `quarterly`
/// Note: Quarters are assumed to be 91 days
fn parse_quarterly<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("quarters", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Parse using any of the known formats
        let (input, duration) = alt((parse_quarterly_ordinal, parse_quarterly_literal))(input)?;
        Ok((input, duration))
    })(input)
}

/// Parse semiannual
///
/// Note: Semiannual is assumed to be 183 days
fn parse_semiannual<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("semiannual", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Optional ordinal
        let (input, digit) = opt(digit1)(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Semiannual literal
        let (input, _) = tag("semiannual")(input)?;
        // Turn into a duration
        Ok((
            input,
            Duration::days(183 * digit.unwrap_or("1").parse::<u32>().unwrap()),
        ))
    })(input)
}

/// Parse annual
///
/// Note: Annual is assumed to be 365 days
fn parse_annual<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("annual", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Optional ordinal
        let (input, digit) = opt(digit1)(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Annual literal
        let (input, _) = tag("annual")(input)?;
        // Turn into a duration
        Ok((
            input,
            Duration::days(365 * digit.unwrap_or("1").parse::<u32>().unwrap()),
        ))
    })(input)
}

/// Parse biannual
///
/// Note: Biannual is assumed to be 730 days
fn parse_biannual<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("biannual", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Optional ordinal
        let (input, digit) = opt(digit1)(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Biannual literal
        let (input, _) = tag("biannual")(input)?;
        // Turn into a duration
        Ok((
            input,
            Duration::days(730 * digit.unwrap_or("1").parse::<u32>().unwrap()),
        ))
    })(input)
}

/// Parse biyearly
fn parse_biyearly<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("biyearly", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Optional ordinal
        let (input, digit) = opt(digit1)(input)?;
        // Any amount of space
        let (input, _) = space0(input)?;
        // Biyearly literal
        let (input, _) = tag("biyearly")(input)?;
        // Turn into a duration
        Ok((
            input,
            Duration::days(730 * digit.unwrap_or("1").parse::<u32>().unwrap()),
        ))
    })(input)
}

/// Combine all the duration format parsers into one
fn parse_duration_duration_format<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("duration", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Parse using any of the known formats
        let (input, duration) = alt((
            parse_sennights,
            parse_seconds,
            parse_minutes,
            parse_hours,
            parse_days,
            parse_weekdays,
            parse_weeks,
            parse_biweekly,
            parse_months,
            parse_bimonthly,
            parse_years,
            parse_quarterly,
            parse_semiannual,
            parse_annual,
            parse_biannual,
            parse_biyearly,
            parse_fortnights,
        ))(input)?;
        Ok((input, duration))
    })(input)
}

/// Parse ISO-8601 duration format
///
/// e.g. `P1Y2M3DT4H5M6S`
fn parse_duration_iso_8601<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("iso-8601", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Literal `P`
        let (input, _) = tag("P")(input)?;
        // Parse the optional year: `1Y`
        let (input, years) = opt({
            map_res(tuple((digit1, tag("Y"))), |(years, _): (&str, &str)| {
                years.parse::<u32>()
            })
        })(input)?;
        // Parse the optional month
        let (input, months) = opt(map_res(
            tuple((digit1, tag("M"))),
            |(years, _): (&str, &str)| years.parse::<u32>(),
        ))(input)?;
        // Parse the optional day
        let (input, days) = opt(map_res(
            tuple((digit1, tag("D"))),
            |(years, _): (&str, &str)| years.parse::<u32>(),
        ))(input)?;

        // Literal `T`
        let (input, _) = opt(tag("T"))(input)?;

        // Parse the optional hour
        let (input, hours) = opt(map_res(
            tuple((digit1, tag("H"))),
            |(years, _): (&str, &str)| years.parse::<u32>(),
        ))(input)?;
        // Parse the optional minute
        let (input, minutes) = opt(map_res(
            tuple((digit1, tag("M"))),
            |(years, _): (&str, &str)| years.parse::<u32>(),
        ))(input)?;
        // Parse the optional second
        let (input, seconds) = opt(map_res(
            tuple((digit1, tag("S"))),
            |(years, _): (&str, &str)| years.parse::<u32>(),
        ))(input)?;

        // Turn into a duration
        Ok((
            input,
            Duration::days(days.unwrap_or(0))
                + Duration::days(years.unwrap_or(0) * 365)
                + Duration::days(months.unwrap_or(0) * 30)
                + Duration::hours(hours.unwrap_or(0))
                + Duration::minutes(minutes.unwrap_or(0))
                + Duration::seconds(seconds.unwrap_or(0)),
        ))
    })(input)
}

/// Combine both duration parsers into one
pub fn parse_duration<'a>(input: &'a str) -> IResult<&'a str, Duration> {
    context("duration", |input: &'a str| {
        // Any amount of space
        let (input, _) = space0(input)?;
        // Parse using any of the known formats
        let (input, duration) =
            alt((parse_duration_iso_8601, parse_duration_duration_format))(input)?;
        Ok((input, duration))
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds() {
        let input = "5 seconds";
        let (input, duration) = parse_seconds(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::seconds(5));

        let input = "5 second";
        let (input, duration) = parse_seconds(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::seconds(5));

        let input = "5 secs";
        let (input, duration) = parse_seconds(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::seconds(5));

        let input = "5 sec";
        let (input, duration) = parse_seconds(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::seconds(5));

        let input = "5 s";
        let (input, duration) = parse_seconds(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::seconds(5));
    }
    #[test]
    fn seconds_spaces() {
        let input = "5seconds";
        let (input, duration) = parse_seconds(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::seconds(5));

        let input = "5          second";
        let (input, duration) = parse_seconds(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::seconds(5));

        let input = "5                     s";
        let (input, duration) = parse_seconds(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::seconds(5));
    }
    #[test]
    fn seconds_no_number() {
        let input = "second";
        let (input, duration) = parse_seconds(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::seconds(1));

        let input = "sec";
        let (input, duration) = parse_seconds(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::seconds(1));
    }
    #[test]
    fn minutes() {
        let input = "5 minutes";
        let (input, duration) = parse_minutes(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::minutes(5));

        let input = "5 minute";
        let (input, duration) = parse_minutes(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::minutes(5));

        let input = "5 mins";
        let (input, duration) = parse_minutes(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::minutes(5));

        let input = "5 min";
        let (input, duration) = parse_minutes(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::minutes(5));
    }
    #[test]
    fn minutes_spaces() {
        let input = "5minutes";
        let (input, duration) = parse_minutes(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::minutes(5));

        let input = "5          minute";
        let (input, duration) = parse_minutes(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::minutes(5));

        let input = "5                     min";
        let (input, duration) = parse_minutes(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::minutes(5));
    }
    #[test]
    fn minutes_no_number() {
        let input = "minute";
        let (input, duration) = parse_minutes(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::minutes(1));

        let input = "min";
        let (input, duration) = parse_minutes(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::minutes(1));
    }
    #[test]
    fn hours() {
        let input = "5 hours";
        let (input, duration) = parse_hours(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::hours(5));

        let input = "5 hour";
        let (input, duration) = parse_hours(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::hours(5));

        let input = "5 hrs";
        let (input, duration) = parse_hours(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::hours(5));

        let input = "5 hr";
        let (input, duration) = parse_hours(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::hours(5));

        let input = "5 h";
        let (input, duration) = parse_hours(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::hours(5));
    }
    #[test]
    fn hours_spaces() {
        let input = "5hours";
        let (input, duration) = parse_hours(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::hours(5));

        let input = "5          hour";
        let (input, duration) = parse_hours(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::hours(5));

        let input = "5                     hr";
        let (input, duration) = parse_hours(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::hours(5));
    }
    #[test]
    fn hours_no_number() {
        let input = "hour";
        let (input, duration) = parse_hours(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::hours(1));

        let input = "hr";
        let (input, duration) = parse_hours(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::hours(1));
    }
    #[test]
    fn days() {
        let input = "5 days";
        let (input, duration) = parse_days(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5));

        let input = "5 day";
        let (input, duration) = parse_days(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5));

        let input = "5 d";
        let (input, duration) = parse_days(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5));

        let input = "5 daily";
        let (input, duration) = parse_days(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5));
    }
    #[test]
    fn days_spaces() {
        let input = "5days";
        let (input, duration) = parse_days(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5));

        let input = "5          day";
        let (input, duration) = parse_days(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5));

        let input = "5                     d";
        let (input, duration) = parse_days(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5));
    }
    #[test]
    fn days_no_number() {
        let input = "daily";
        let (input, duration) = parse_days(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(1));

        let input = "day";
        let (input, duration) = parse_days(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(1));
    }
    #[test]
    fn weeks() {
        let input = "5 weeks";
        let (input, duration) = parse_weeks(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::weeks(5));

        let input = "5 week";
        let (input, duration) = parse_weeks(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::weeks(5));

        let input = "5 w";
        let (input, duration) = parse_weeks(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::weeks(5));

        let input = "5 weekly";
        let (input, duration) = parse_weeks(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::weeks(5));
    }
    #[test]
    fn weeks_spaces() {
        let input = "5weeks";
        let (input, duration) = parse_weeks(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::weeks(5));

        let input = "5          week";
        let (input, duration) = parse_weeks(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::weeks(5));

        let input = "5                     w";
        let (input, duration) = parse_weeks(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::weeks(5));
    }
    #[test]
    fn weeks_no_number() {
        let input = "weekly";
        let (input, duration) = parse_weeks(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::weeks(1));

        let input = "week";
        let (input, duration) = parse_weeks(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::weeks(1));

        let input = "wk";
        let (input, duration) = parse_weeks(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::weeks(1));
    }
    #[test]
    fn months() {
        let input = "5 months";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 30));

        let input = "5 month";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 30));

        let input = "5 m";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 30));

        let input = "5 monthly";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 30));
    }
    #[test]
    fn months_spaces() {
        let input = "5months";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 30));

        let input = "5          month";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 30));

        let input = "5                     m";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 30));
    }
    #[test]
    fn months_no_number() {
        let input = "monthly";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(30));

        let input = "month";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(30));

        let input = "mth";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(30));

        let input = "mo";
        let (input, duration) = parse_months(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(30));
    }
    #[test]
    fn years() {
        let input = "5 years";
        let (input, duration) = parse_years(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 365));

        let input = "5 year";
        let (input, duration) = parse_years(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 365));

        let input = "5 y";
        let (input, duration) = parse_years(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 365));

        let input = "5 yearly";
        let (input, duration) = parse_years(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 365));
    }
    #[test]
    fn years_spaces() {
        let input = "5years";
        let (input, duration) = parse_years(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 365));

        let input = "5          year";
        let (input, duration) = parse_years(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 365));

        let input = "5                     y";
        let (input, duration) = parse_years(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 365));
    }
    #[test]
    fn years_no_number() {
        let input = "yearly";
        let (input, duration) = parse_years(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(365));

        let input = "year";
        let (input, duration) = parse_years(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(365));

        let input = "yr";
        let (input, duration) = parse_years(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(365));
    }
    /// A special recurring duration that is equivalent to "P1D" but should serialize to "weekdays"
    /// when no ordinal is present.
    #[test]
    fn weekdays() {
        let input = "weekdays";
        let (input, duration) = parse_weekdays(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(1));
        assert_eq!(duration.to_string(), "weekdays".to_string());
        // After any math, it should revert to the ISO format.
        assert_eq!((duration.clone() + duration.clone()).to_string(), "P2D".to_string());

        let input = "5    weekdays";
        let (input, duration) = parse_weekdays(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5));
    }
    #[test]
    fn fortnights() {
        let input = "fortnight";
        let (input, duration) = parse_fortnights(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(14));

        let input = "5    fortnight";
        let (input, duration) = parse_fortnights(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 14));
    }
    #[test]
    fn sennights() {
        let input = "sennight";
        let (input, duration) = parse_sennights(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(7));

        let input = "5    sennight";
        let (input, duration) = parse_sennights(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 7));
    }
    #[test]
    fn biweekly() {
        let input = "biweekly";
        let (input, duration) = parse_biweekly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(14));

        let input = "5    biweekly";
        let (input, duration) = parse_biweekly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 14));
    }
    #[test]
    fn bimonthly() {
        let input = "bimonthly";
        let (input, duration) = parse_bimonthly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(61));

        let input = "5    bimonthly";
        let (input, duration) = parse_bimonthly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 61));
    }
    #[test]
    fn quarterly() {
        let input = "5quarterly";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5quarters";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5quarter";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5qrtrs";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5qrtr";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5qtr";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5q";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));
    }
    #[test]
    fn quarterly_spaces() {
        let input = "5     quarterly";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5     quarters";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5     quarter";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5     qrtrs";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5     qrtr";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5     qtr";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));

        let input = "5     q";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 91));
    }
    #[test]
    fn quarterly_no_number() {
        let input = "quarterly";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(91));

        let input = "quarter";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(91));

        let input = "qrtr";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(91));

        let input = "qtr";
        let (input, duration) = parse_quarterly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(91));
    }
    #[test]
    fn semiannual() {
        let input = "semiannual";
        let (input, duration) = parse_semiannual(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(183));

        let input = "5semiannual";
        let (input, duration) = parse_semiannual(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 183));

        let input = "5    semiannual";
        let (input, duration) = parse_semiannual(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 183));
    }
    #[test]
    fn annual() {
        let input = "annual";
        let (input, duration) = parse_annual(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(365));

        let input = "5annual";
        let (input, duration) = parse_annual(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 365));

        let input = "5    annual";
        let (input, duration) = parse_annual(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 365));
    }
    #[test]
    fn biannual() {
        let input = "biannual";
        let (input, duration) = parse_biannual(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(2 * 365));

        let input = "5biannual";
        let (input, duration) = parse_biannual(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 2 * 365));

        let input = "5    biannual";
        let (input, duration) = parse_biannual(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 2 * 365));
    }
    #[test]
    fn biyearly() {
        let input = "biyearly";
        let (input, duration) = parse_biyearly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(2 * 365));

        let input = "5biyearly";
        let (input, duration) = parse_biyearly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 2 * 365));

        let input = "5    biyearly";
        let (input, duration) = parse_biyearly(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(duration, Duration::days(5 * 2 * 365));
    }
    /// Test the aggregate duration format parser
    ///
    /// Uses the list from the official docs to ensure the parser is correct:
    /// <https://taskwarrior.org/docs/durations/>
    #[test]
    fn duration_duration_format() {
        assert_eq!(
            parse_duration_duration_format("5 seconds").unwrap().1,
            Duration::seconds(5)
        );
        assert_eq!(
            parse_duration_duration_format("5 second").unwrap().1,
            Duration::seconds(5)
        );
        assert_eq!(
            parse_duration_duration_format("5 secs").unwrap().1,
            Duration::seconds(5)
        );
        assert_eq!(
            parse_duration_duration_format("5 sec").unwrap().1,
            Duration::seconds(5)
        );
        assert_eq!(
            parse_duration_duration_format("5 s").unwrap().1,
            Duration::seconds(5)
        );
        assert_eq!(
            parse_duration_duration_format("5seconds").unwrap().1,
            Duration::seconds(5)
        );
        assert_eq!(
            parse_duration_duration_format("5second").unwrap().1,
            Duration::seconds(5)
        );
        assert_eq!(
            parse_duration_duration_format("5secs").unwrap().1,
            Duration::seconds(5)
        );
        assert_eq!(
            parse_duration_duration_format("5sec").unwrap().1,
            Duration::seconds(5)
        );
        assert_eq!(
            parse_duration_duration_format("5s").unwrap().1,
            Duration::seconds(5)
        );
        assert_eq!(
            parse_duration_duration_format("second").unwrap().1,
            Duration::seconds(1)
        );
        assert_eq!(
            parse_duration_duration_format("sec").unwrap().1,
            Duration::seconds(1)
        );
        assert_eq!(
            parse_duration_duration_format("5 minutes").unwrap().1,
            Duration::minutes(5)
        );
        assert_eq!(
            parse_duration_duration_format("5 minute").unwrap().1,
            Duration::minutes(5)
        );
        assert_eq!(
            parse_duration_duration_format("5 mins").unwrap().1,
            Duration::minutes(5)
        );
        assert_eq!(
            parse_duration_duration_format("5 min").unwrap().1,
            Duration::minutes(5)
        );
        assert_eq!(
            parse_duration_duration_format("5minutes").unwrap().1,
            Duration::minutes(5)
        );
        assert_eq!(
            parse_duration_duration_format("5minute").unwrap().1,
            Duration::minutes(5)
        );
        assert_eq!(
            parse_duration_duration_format("5mins").unwrap().1,
            Duration::minutes(5)
        );
        assert_eq!(
            parse_duration_duration_format("5min").unwrap().1,
            Duration::minutes(5)
        );
        assert_eq!(
            parse_duration_duration_format("minute").unwrap().1,
            Duration::minutes(1)
        );
        assert_eq!(
            parse_duration_duration_format("min").unwrap().1,
            Duration::minutes(1)
        );
        assert_eq!(
            parse_duration_duration_format("3 hours").unwrap().1,
            Duration::hours(3)
        );
        assert_eq!(
            parse_duration_duration_format("3 hour").unwrap().1,
            Duration::hours(3)
        );
        assert_eq!(
            parse_duration_duration_format("3 hrs").unwrap().1,
            Duration::hours(3)
        );
        assert_eq!(
            parse_duration_duration_format("3 hr").unwrap().1,
            Duration::hours(3)
        );
        assert_eq!(
            parse_duration_duration_format("3 h").unwrap().1,
            Duration::hours(3)
        );
        assert_eq!(
            parse_duration_duration_format("3hours").unwrap().1,
            Duration::hours(3)
        );
        assert_eq!(
            parse_duration_duration_format("3hour").unwrap().1,
            Duration::hours(3)
        );
        assert_eq!(
            parse_duration_duration_format("3hrs").unwrap().1,
            Duration::hours(3)
        );
        assert_eq!(
            parse_duration_duration_format("3hr").unwrap().1,
            Duration::hours(3)
        );
        assert_eq!(
            parse_duration_duration_format("3h").unwrap().1,
            Duration::hours(3)
        );
        assert_eq!(
            parse_duration_duration_format("hour").unwrap().1,
            Duration::hours(1)
        );
        assert_eq!(
            parse_duration_duration_format("hr").unwrap().1,
            Duration::hours(1)
        );
        assert_eq!(
            parse_duration_duration_format("2 days").unwrap().1,
            Duration::days(2)
        );
        assert_eq!(
            parse_duration_duration_format("2 day").unwrap().1,
            Duration::days(2)
        );
        assert_eq!(
            parse_duration_duration_format("2 d").unwrap().1,
            Duration::days(2)
        );
        assert_eq!(
            parse_duration_duration_format("2days").unwrap().1,
            Duration::days(2)
        );
        assert_eq!(
            parse_duration_duration_format("2day").unwrap().1,
            Duration::days(2)
        );
        assert_eq!(
            parse_duration_duration_format("2d").unwrap().1,
            Duration::days(2)
        );
        assert_eq!(
            parse_duration_duration_format("daily").unwrap().1,
            Duration::days(1)
        );
        assert_eq!(
            parse_duration_duration_format("day").unwrap().1,
            Duration::days(1)
        );
        assert_eq!(
            parse_duration_duration_format("3 weeks").unwrap().1,
            Duration::weeks(3)
        );
        assert_eq!(
            parse_duration_duration_format("3 week").unwrap().1,
            Duration::weeks(3)
        );
        assert_eq!(
            parse_duration_duration_format("3 wks").unwrap().1,
            Duration::weeks(3)
        );
        assert_eq!(
            parse_duration_duration_format("3 wk").unwrap().1,
            Duration::weeks(3)
        );
        assert_eq!(
            parse_duration_duration_format("3 w").unwrap().1,
            Duration::weeks(3)
        );
        assert_eq!(
            parse_duration_duration_format("3weeks").unwrap().1,
            Duration::weeks(3)
        );
        assert_eq!(
            parse_duration_duration_format("3week").unwrap().1,
            Duration::weeks(3)
        );
        assert_eq!(
            parse_duration_duration_format("3wks").unwrap().1,
            Duration::weeks(3)
        );
        assert_eq!(
            parse_duration_duration_format("3wk").unwrap().1,
            Duration::weeks(3)
        );
        assert_eq!(
            parse_duration_duration_format("3w").unwrap().1,
            Duration::weeks(3)
        );
        assert_eq!(
            parse_duration_duration_format("weekly").unwrap().1,
            Duration::weeks(1)
        );
        assert_eq!(
            parse_duration_duration_format("week").unwrap().1,
            Duration::weeks(1)
        );
        assert_eq!(
            parse_duration_duration_format("wk").unwrap().1,
            Duration::weeks(1)
        );
        assert_eq!(
            parse_duration_duration_format("weekdays").unwrap().1,
            Duration::days(1)
        );
        assert_eq!(
            parse_duration_duration_format("2 fortnight").unwrap().1,
            Duration::days(28)
        );
        assert_eq!(
            parse_duration_duration_format("2 sennight").unwrap().1,
            Duration::days(14)
        );
        assert_eq!(
            parse_duration_duration_format("2fortnight").unwrap().1,
            Duration::days(28)
        );
        assert_eq!(
            parse_duration_duration_format("2sennight").unwrap().1,
            Duration::days(14)
        );
        assert_eq!(
            parse_duration_duration_format("biweekly").unwrap().1,
            Duration::days(14)
        );
        assert_eq!(
            parse_duration_duration_format("fortnight").unwrap().1,
            Duration::days(14)
        );
        assert_eq!(
            parse_duration_duration_format("sennight").unwrap().1,
            Duration::days(7)
        );
        assert_eq!(
            parse_duration_duration_format("5 months").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5 month").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5 mnths").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5 mths").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5 mth").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5 mo").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5 m").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5months").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5month").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5mnths").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5mths").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5mth").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5mo").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("5m").unwrap().1,
            Duration::days(150)
        );
        assert_eq!(
            parse_duration_duration_format("monthly").unwrap().1,
            Duration::days(30)
        );
        assert_eq!(
            parse_duration_duration_format("month").unwrap().1,
            Duration::days(30)
        );
        assert_eq!(
            parse_duration_duration_format("mo").unwrap().1,
            Duration::days(30)
        );
        assert_eq!(
            parse_duration_duration_format("bimonthly").unwrap().1,
            Duration::days(61)
        );
        assert_eq!(
            parse_duration_duration_format("1 quarterly").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1 quarters").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1 quarter").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1 qrtrs").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1 qrtr").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1 qtr").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1 q").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1quarterly").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1quarters").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1quarter").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1qrtrs").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1qrtr").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1qtr").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("1q").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("quarterly").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("quarter").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("qrtr").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("qtr").unwrap().1,
            Duration::days(91)
        );
        assert_eq!(
            parse_duration_duration_format("semiannual").unwrap().1,
            Duration::days(183)
        );
        assert_eq!(
            parse_duration_duration_format("1 years").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("1 year").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("1 yrs").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("1 yr").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("1 y").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("1years").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("1year").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("1yrs").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("1yr").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("1y").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("annual").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("yearly").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("year").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("yr").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_duration_format("biannual").unwrap().1,
            Duration::days(730)
        );
        assert_eq!(
            parse_duration_duration_format("biyearly").unwrap().1,
            Duration::days(730)
        );
    }
    /// Test that the ISO 8601 duration parser works as expected.
    ///
    /// Use the following as a reference: <https://taskwarrior.org/docs/durations/>
    #[test]
    fn iso_8601() {
        assert_eq!(
            parse_duration_iso_8601("P1Y").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_iso_8601("P1M").unwrap().1,
            Duration::days(30)
        );
        assert_eq!(parse_duration_iso_8601("P1D").unwrap().1, Duration::days(1));
        assert_eq!(
            parse_duration_iso_8601("P1Y2M").unwrap().1,
            Duration::days(425)
        );
        assert_eq!(
            parse_duration_iso_8601("P1Y").unwrap().1,
            Duration::days(365)
        );
        assert_eq!(
            parse_duration_iso_8601("P1M").unwrap().1,
            Duration::days(30)
        );
        assert_eq!(parse_duration_iso_8601("P1D").unwrap().1, Duration::days(1));
        assert_eq!(
            parse_duration_iso_8601("PT5H6M7S").unwrap().1,
            Duration::hours(5) + Duration::minutes(6) + Duration::seconds(7)
        );
        assert_eq!(
            parse_duration_iso_8601("PT12H40M50S").unwrap().1,
            Duration::hours(12) + Duration::minutes(40) + Duration::seconds(50)
        );
        assert_eq!(
            parse_duration_iso_8601("P1Y2M3DT12H40M50S").unwrap().1,
            Duration::days(365)
                + Duration::days(2 * 30)
                + Duration::days(3)
                + Duration::hours(12)
                + Duration::minutes(40)
                + Duration::seconds(50)
        );
    }
    /// Test the forward-facing parse.
    #[test]
    fn duration() {
        // Some duration formats
        assert_eq!(parse_duration("1 year").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("1 years").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("1 yrs").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("1 yr").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("1 y").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("1year").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("1years").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("1yrs").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("1yr").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("1y").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("yearly").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("annual").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("year").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("yr").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("monthly").unwrap().1, Duration::days(30));
        assert_eq!(parse_duration("month").unwrap().1, Duration::days(30));
        assert_eq!(parse_duration("mo").unwrap().1, Duration::days(30));
        assert_eq!(parse_duration("bimonthly").unwrap().1, Duration::days(61));
        assert_eq!(parse_duration("1 quarterly").unwrap().1, Duration::days(91));
        assert_eq!(parse_duration("1 quarters").unwrap().1, Duration::days(91));
        assert_eq!(parse_duration("1 quarter").unwrap().1, Duration::days(91));
        assert_eq!(parse_duration("1 qrtrs").unwrap().1, Duration::days(91));
        assert_eq!(parse_duration("1 qrtr").unwrap().1, Duration::days(91));
        assert_eq!(parse_duration("1 qtr").unwrap().1, Duration::days(91));
        // Some ISO 8601 formats
        assert_eq!(parse_duration("P1Y").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("P1M").unwrap().1, Duration::days(30));
        assert_eq!(parse_duration("P1D").unwrap().1, Duration::days(1));
        assert_eq!(parse_duration("P1Y2M").unwrap().1, Duration::days(425));
        assert_eq!(parse_duration("P1Y").unwrap().1, Duration::days(365));
        assert_eq!(parse_duration("P1M").unwrap().1, Duration::days(30));
        assert_eq!(parse_duration("P1D").unwrap().1, Duration::days(1));
        assert_eq!(
            parse_duration("PT5H6M7S").unwrap().1,
            Duration::hours(5) + Duration::minutes(6) + Duration::seconds(7)
        );
        assert_eq!(
            parse_duration("PT12H40M50S").unwrap().1,
            Duration::hours(12) + Duration::minutes(40) + Duration::seconds(50)
        );
        assert_eq!(
            parse_duration("P1Y2M3DT12H40M50S").unwrap().1,
            Duration::days(365)
                + Duration::days(2 * 30)
                + Duration::days(3)
                + Duration::hours(12)
                + Duration::minutes(40)
                + Duration::seconds(50)
        );
    }
    #[test]
    fn zero() {
        let input = "PT0S";
        // Just make sure it parses
        let _duration: Duration = input.into();
    }
    #[test]
    /// Ensure converted string values are smoothed.
    ///
    /// e.g. Duration::seconds(7200) -> PT2H
    fn smoothing() {
        use chrono::{self, offset::Utc, DateTime, TimeZone};

        let duration: Duration = Duration::hours(1) + Duration::hours(1);
        let string = duration.to_string();
        assert_eq!(&string, "PT2H");

        let start: DateTime<Utc> = Utc.with_ymd_and_hms(2020, 1, 1, 12, 0, 0).unwrap();
        let end: DateTime<Utc> = Utc.with_ymd_and_hms(2020, 1, 1, 14, 0, 0).unwrap();

        let duration = end.signed_duration_since(start);
        let mut elapsed: Duration = duration.into();
        elapsed.smooth();

        assert_eq!(&elapsed.to_string(), "PT2H");
    }
    /// Verify that the deserialization matches the serialization, unless math is done.
    #[test]
    fn source() {
        let input = "P1M";
        let duration: Duration = input.into();
        assert_eq!(duration, Duration::months(1));
        assert_eq!(duration.to_string(), "P1M".to_string());
        // After any math, it should remove the source.
        // In this case, it smooths 1 month + 1 month to 60 days
        assert_eq!((duration.clone() + duration.clone()).to_string(), "P60D".to_string());
    }
}
