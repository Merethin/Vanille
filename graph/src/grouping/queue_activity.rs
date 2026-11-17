use chrono::{DateTime, Timelike};
use itertools::Itertools;
use vanille_models::report::ReportEntry;
use crate::bins::CATEGORIES;

pub fn group_data(
    entries: &[ReportEntry]
) -> Vec<Vec<i64>> {
    let mut grouped = entries.iter().map(|entry| {
        let datetime = DateTime::from_timestamp(entry.queue_time, 0).expect("Invalid timestamp");
        let hour = datetime.hour();
        let category = &entry.event;

        (hour, category)
    }).into_group_map_by(|a| a.0);

    for hour in 0..24 {
        grouped.entry(hour).or_default();
    }

    let mut values = Vec::new();

    for _ in 0..CATEGORIES.len() {
        values.push(Vec::<i64>::new());
    }

    for (_, data) in grouped.iter().sorted_by_key(|v| v.0) {
        for (index, category) in CATEGORIES.iter().enumerate() {
            values[index].push(
                data.iter().map(|v| v.1).filter(|t| t == category).count() as i64
            );
        }
    }

    values
}
