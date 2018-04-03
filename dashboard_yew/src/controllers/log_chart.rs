use models::*;
use chrono::prelude::*;
use time::Duration;
use std::ops::Sub;

pub fn create_log_chart_values(logs:Vec<LogByUser>) -> Vec<u64> {
    // log chart
    // last 30m
    let thirty_min_ago = Utc::now().sub(Duration::minutes(30)).timestamp();
    let recent_logs:Vec<LogByUser> = logs.into_iter().filter(|l| (l.log.unix_seconds as i64) > thirty_min_ago).collect();

    let mut counts:Vec<u64> = Vec::new();

    // extra 5 for good measure(read: chart appearance hack)
    let interval:i64 = 5;
    let marks:Vec<i64> = vec![30,25,20,15,10,5,5];
    for min_mark in marks {
        let start_min_ago = Utc::now().sub(Duration::minutes(min_mark)).timestamp();
        let end_min_ago = Utc::now().sub(Duration::minutes(min_mark - interval)).timestamp();

        let count = recent_logs.iter().filter(|l|   (l.log.unix_seconds as i64) < end_min_ago &&
                                                    (l.log.unix_seconds as i64)  > start_min_ago).count() as u64;
        counts.push(count);
    }
    return counts.clone();
}