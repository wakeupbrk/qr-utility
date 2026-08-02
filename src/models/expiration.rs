use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeUnit {
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
}

impl fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeUnit::Minutes => write!(f, "Minutes"),
            TimeUnit::Hours => write!(f, "Hours"),
            TimeUnit::Days => write!(f, "Days"),
            TimeUnit::Weeks => write!(f, "Weeks"),
            TimeUnit::Months => write!(f, "Months"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpirationOption {
    FiveMinutes,
    TenMinutes,
    ThirtyMinutes,
    OneHour,
    OneDay,
    SevenDays,
    Never,
    Custom { value: u64, unit: TimeUnit },
}

impl ExpirationOption {
    pub const ALL_PRESETS: &'static [ExpirationOption] = &[
        ExpirationOption::FiveMinutes,
        ExpirationOption::TenMinutes,
        ExpirationOption::ThirtyMinutes,
        ExpirationOption::OneHour,
        ExpirationOption::OneDay,
        ExpirationOption::SevenDays,
        ExpirationOption::Never,
    ];

    pub fn label(&self) -> String {
        match self {
            ExpirationOption::FiveMinutes => "5 minutes".to_string(),
            ExpirationOption::TenMinutes => "10 minutes".to_string(),
            ExpirationOption::ThirtyMinutes => "30 minutes".to_string(),
            ExpirationOption::OneHour => "1 hour".to_string(),
            ExpirationOption::OneDay => "1 day".to_string(),
            ExpirationOption::SevenDays => "7 days".to_string(),
            ExpirationOption::Never => "Never expires".to_string(),
            ExpirationOption::Custom { value, unit } => format!("Custom ({} {})", value, unit),
        }
    }

    /// Calculate the target expiration timestamp from now, if applicable.
    pub fn to_timestamp(&self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        match self {
            ExpirationOption::FiveMinutes => Some(now + Duration::minutes(5)),
            ExpirationOption::TenMinutes => Some(now + Duration::minutes(10)),
            ExpirationOption::ThirtyMinutes => Some(now + Duration::minutes(30)),
            ExpirationOption::OneHour => Some(now + Duration::hours(1)),
            ExpirationOption::OneDay => Some(now + Duration::days(1)),
            ExpirationOption::SevenDays => Some(now + Duration::days(7)),
            ExpirationOption::Never => None,
            ExpirationOption::Custom { value, unit } => {
                let v = *value as i64;
                let duration = match unit {
                    TimeUnit::Minutes => Duration::minutes(v),
                    TimeUnit::Hours => Duration::hours(v),
                    TimeUnit::Days => Duration::days(v),
                    TimeUnit::Weeks => Duration::weeks(v),
                    TimeUnit::Months => Duration::days(v * 30),
                };
                Some(now + duration)
            }
        }
    }

    /// Check if a given timestamp has expired relative to UTC now.
    pub fn is_expired(expiration_time: Option<DateTime<Utc>>) -> bool {
        match expiration_time {
            Some(exp) => Utc::now() >= exp,
            None => false,
        }
    }

    /// Format remaining time until expiration in human-readable format.
    pub fn format_remaining(expiration_time: Option<DateTime<Utc>>) -> String {
        match expiration_time {
            None => "Never expires".to_string(),
            Some(exp) => {
                let now = Utc::now();
                if now >= exp {
                    "Expired".to_string()
                } else {
                    let diff = exp - now;
                    if diff.num_days() > 0 {
                        format!("Expires in {}d {}h", diff.num_days(), diff.num_hours() % 24)
                    } else if diff.num_hours() > 0 {
                        format!(
                            "Expires in {}h {}m",
                            diff.num_hours(),
                            diff.num_minutes() % 60
                        )
                    } else if diff.num_minutes() > 0 {
                        format!(
                            "Expires in {}m {}s",
                            diff.num_minutes(),
                            diff.num_seconds() % 60
                        )
                    } else {
                        format!("Expires in {}s", diff.num_seconds())
                    }
                }
            }
        }
    }

    /// Parse simple string like "5m", "10m", "1h", "1d", "7d", "never" into ExpirationOption.
    pub fn from_str_lenient(s: &str) -> Result<Self, String> {
        let s_lower = s.trim().to_lowercase();
        if s_lower == "never" || s_lower == "none" || s_lower == "0" {
            return Ok(ExpirationOption::Never);
        }
        if s_lower == "5m" || s_lower == "5min" {
            return Ok(ExpirationOption::FiveMinutes);
        }
        if s_lower == "10m" || s_lower == "10min" {
            return Ok(ExpirationOption::TenMinutes);
        }
        if s_lower == "30m" || s_lower == "30min" {
            return Ok(ExpirationOption::ThirtyMinutes);
        }
        if s_lower == "1h" || s_lower == "1hour" {
            return Ok(ExpirationOption::OneHour);
        }
        if s_lower == "1d" || s_lower == "1day" {
            return Ok(ExpirationOption::OneDay);
        }
        if s_lower == "7d" || s_lower == "7days" {
            return Ok(ExpirationOption::SevenDays);
        }

        // Try parsing custom e.g. "45m", "12h", "3w", "2mo"
        let len = s_lower.len();
        if len > 1 {
            let (num_part, unit_part) = s_lower.split_at(len - 1);
            if let Ok(num) = num_part.parse::<u64>() {
                let unit = match unit_part {
                    "m" => TimeUnit::Minutes,
                    "h" => TimeUnit::Hours,
                    "d" => TimeUnit::Days,
                    "w" => TimeUnit::Weeks,
                    "M" => TimeUnit::Months,
                    _ => return Err(format!("Unknown time unit: {}", unit_part)),
                };
                return Ok(ExpirationOption::Custom { value: num, unit });
            }
        }

        Err(format!(
            "Invalid expiration format: '{}'. Try e.g. '30m', '1h', '7d', or 'never'.",
            s
        ))
    }
}
