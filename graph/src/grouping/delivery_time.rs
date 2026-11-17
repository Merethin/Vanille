use chrono::{DateTime, Timelike};
use itertools::Itertools;
use vanille_models::report::ReportEntry;
use crate::bins::INTERVALS;

pub fn group_data(
    entries: &[ReportEntry]
) -> Vec<Vec<i64>> {
    let mut grouped = entries.iter().map(|entry| {
        let datetime = DateTime::from_timestamp(entry.sent_time, 0).expect("Invalid timestamp");
        let hour = datetime.hour();
        let interval = entry.sent_time - entry.queue_time;

        (hour, interval)
    }).into_group_map_by(|a| a.0);

    for hour in 0..24 {
        grouped.entry(hour).or_default();
    }

    let mut values = Vec::new();

    for _ in 0..INTERVALS.len() {
        values.push(Vec::<i64>::new());
    }

    for (_, data) in grouped.iter().sorted_by_key(|v| v.0) {
        for (index, range) in INTERVALS.iter().enumerate() {
            values[index].push(
                data.iter().map(|v| v.1 as f64).filter(|t| range.contains(t)).count() as i64
            );
        }
    }

    values
}
