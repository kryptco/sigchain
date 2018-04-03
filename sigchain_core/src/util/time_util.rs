use super::chrono::prelude::*;
use std::convert::From;
use super::time::Duration;
use std::ops::Sub;

pub trait TimeAgo {
    fn time_ago(self) -> String;
    fn full_timestamp(self) -> String;
}

impl TimeAgo for u64 {
    fn time_ago(self) -> String {
        (self as i64).time_ago()
    }
    fn full_timestamp(self) -> String { (self as i64).full_timestamp() }
}

impl TimeAgo for i64 {
    fn time_ago(self) -> String {
        let current_time_seconds = Duration::seconds(i64::from(Utc::now().naive_utc().timestamp()));
        let duration = current_time_seconds.sub(Duration::seconds(self));

        let suffix = "ago";

        if duration.num_minutes() < 1 {
            return format!("{}s {}", duration.num_seconds(), suffix);
        } else if duration.num_hours() < 1 {
            return format!("{}m {}", duration.num_minutes(), suffix);
        } else if duration.num_days() < 1 {
            let remainder = duration.sub(Duration::hours(duration.num_hours())).num_minutes();
            return format!("{}h {}m {}", duration.num_hours(), remainder, suffix);
        } else if duration.num_days() < 7 {
            return format!("{} days {}", duration.num_days(), suffix);
        }

        self.full_timestamp()
    }

    fn full_timestamp(self) -> String {
        let event_time = Local.from_utc_datetime(&NaiveDateTime::from_timestamp(self, 0));
        return event_time.format("%l:%M%P on %m/%d/%y ").to_string()
    }

}
