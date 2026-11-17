use std::collections::HashMap;
use itertools::Itertools;
use vanille_models::report::ReportEntry;
use crate::bins::CATEGORIES;

pub fn group_data(
    entries: &[ReportEntry]
) -> (Vec<Vec<i64>>, Vec<String>) {
    let grouped = entries.iter().map(|entry| {
        let sender = &entry.sender;
        let category = &entry.event;

        (sender, category)
    }).into_group_map_by(|a| a.0);

    let mut values: HashMap<&String, Vec<i64>> = HashMap::new();
    let mut totals: HashMap<&String, i64> = HashMap::new();

    for name in grouped.keys() {
        values.entry(name).or_default();
        totals.entry(name).or_default();
    }

    for (name, data) in grouped.iter() {
        for category in CATEGORIES.iter() {
            let count = data.iter().filter(|&t| t.1 == category).count() as i64;
            values.get_mut(name).unwrap().push(count);
            *totals.get_mut(name).unwrap() += count;
        }
    }

    let mut result: Vec<Vec<i64>> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    for _ in 0..CATEGORIES.len() {
        result.push(Vec::<i64>::new());
    }

    for (key, _) in totals.into_iter().sorted_by_key(|(_, v)| *v).rev() {
        for index in 0..CATEGORIES.len() {
            result[index].push(values[key][index]);
        }
        names.push(key.clone());
    }

    (result, names)
}
