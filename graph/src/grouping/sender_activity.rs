use std::{collections::HashMap};
use chrono::{DateTime, Timelike};
use itertools::Itertools;
use vanille_models::report::ReportEntry;

pub fn group_data(
    entries: &[ReportEntry]
) -> (Vec<Vec<i64>>, Vec<String>) {
    let mut grouped = entries.iter().map(|entry| {
        let datetime = DateTime::from_timestamp(entry.sent_time, 0).expect("Invalid timestamp");
        let hour = datetime.hour();
        let sender = &entry.sender;

        (hour, sender)
    }).into_group_map_by(|a| a.0);

    let senders = entries.iter().map(|v| &v.sender).sorted().dedup().collect_vec();

    for hour in 0..24 {
        grouped.entry(hour).or_default();
    }

    let mut values: HashMap<&String, Vec<i64>> = HashMap::new();
    let mut totals: HashMap<&String, i64> = HashMap::new();

    for name in &senders {
        values.entry(name).or_default();
        totals.entry(name).or_default();
    }

    for (_, data) in grouped.iter().sorted_by_key(|v| v.0) {
        for &name in &senders {
            let count = data.iter().filter(|&t| t.1 == name).count() as i64;

            values.get_mut(name).unwrap().push(count);
            *totals.get_mut(name).unwrap() += count;
        }
    }

    let mut result: Vec<Vec<i64>> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    for (key, _) in totals.into_iter().sorted_by_key(|(_, v)| *v).rev() {
        result.push(values[key].clone());
        names.push(key.clone());
    }

    (result, names)
}
