use vanille_models::report::ReportEntry;
use crate::bins::{INTERVALS, CATEGORIES};

pub fn group_data(
    entries: &[ReportEntry]
) -> Vec<Vec<i64>> {
    let grouped = entries.iter().filter_map(|entry| {
        if !entry.moved { return None; }

        let interval = entry.sent_time - entry.queue_time;
        let category = &entry.event;

        Some((interval, category))
    });

    let mut values = Vec::new();

    for _ in 0..CATEGORIES.len() {
        values.push(Vec::<i64>::new());
    }

    for (_, range) in INTERVALS.iter().enumerate() {
        for (index, name) in CATEGORIES.iter().enumerate() {
            values[index].push(
                grouped.clone().filter(|t| range.contains(&(t.0 as f64)) && t.1 == name).count() as i64
            );
        }
    }

    values
}
