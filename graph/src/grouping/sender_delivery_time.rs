use std::collections::HashMap;
use itertools::Itertools;
use vanille_models::report::ReportEntry;
use crate::bins::INTERVALS;

pub fn group_data(
    entries: &[ReportEntry]
) -> (Vec<Vec<i64>>, Vec<String>) {
    let grouped = entries.iter().map(|entry| {
        let sender = &entry.sender;
        let interval = entry.sent_time - entry.queue_time;

        (sender, interval)
    }).into_group_map_by(|a| a.0);

    let mut values: HashMap<&String, Vec<i64>> = HashMap::new();
    let mut totals: HashMap<&String, i64> = HashMap::new();

    for name in grouped.keys() {
        values.entry(name).or_default();
        totals.entry(name).or_default();
    }

    for (name, data) in grouped.iter() {
        for range in INTERVALS.iter() {
            let count = data.iter().filter(|&t| range.contains(&(t.1 as f64))).count() as i64;
            values.get_mut(name).unwrap().push(count);
            *totals.get_mut(name).unwrap() += count;
        }
    }

    let mut result: Vec<Vec<i64>> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    for _ in 0..INTERVALS.len() {
        result.push(Vec::<i64>::new());
    }

    for (key, _) in totals.into_iter().sorted_by_key(|(_, v)| *v).rev() {
        for index in 0..INTERVALS.len() {
            result[index].push(values[key][index]);
        }
        names.push(key.clone());
    }

    (result, names)
}
