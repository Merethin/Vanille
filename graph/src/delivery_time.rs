use std::{error::Error, fs, iter::zip, ops::Range};
use chrono::{DateTime, Timelike};
use itertools::Itertools;
use tempfile::TempDir;
use plotpy::{Barplot, Legend, Plot};

use vanille_models::report::ReportEntry;
use caramel::ns::format::prettify_name;

const BINS: [Range<f64>; 5] = [
    (0.0..60.0),
    (60.0..300.0),
    (300.0..600.0),
    (600.0..1800.0),
    (1800.0..f64::INFINITY)
];

const COLORS: [&'static str; 5] = [
    "#2ca02c", "#1f77b4", "#ff7f0e", "#d62728", "#9467bd"
];

const LEGEND_LABELS: [&'static str; 5] = [
    "<1 min",
    "1-5 min",
    "5-10 min",
    "10-30 min",
    ">30 min"
];

fn group_delivery_time_data(
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

    for _ in 0..BINS.len() {
        values.push(Vec::<i64>::new());
    }

    for (_, data) in grouped.iter().sorted_by_key(|v| v.0) {
        for (index, range) in BINS.iter().enumerate() {
            values[index].push(
                data.iter().map(|v| v.1 as f64).filter(|t| range.contains(t)).count() as i64
            );
        }
    }

    values
}

pub async fn generate_delivery_time_plot(
    region: &str,
    entries: &[ReportEntry]
) -> Result<Vec<u8>, Box<dyn Error>> {
    let values = group_delivery_time_data(entries);

    let tmp_dir = TempDir::new()?;
    let output_path = tmp_dir.path().join("graph.png");

    let mut bar = Barplot::new();
    let mut bottom = vec![0.0; 24];
    let x_labels: Vec<_> = (0..24).map(|hour| format!("{:02}:00", hour)).collect();
    for (index, (label, col)) in zip(LEGEND_LABELS, COLORS).enumerate() {
        let vals = &values[index];
        bar.set_label(
            label
        ).set_colors(
            &[col]
        ).set_bottom(
            &bottom
        ).draw_with_str(
            &x_labels.iter().map(|s| s.as_str()).collect_vec(), vals
        );

        for i in 0..24 {
            bottom[i] += vals[i] as f64;
        }
    }

    let mut legend = Legend::new();
    legend.set_location("upper left");
    legend.draw();

    Plot::new().add(&bar).add(&legend).set_figure_size_inches(
        12f64, 6f64
    ).grid_labels_legend(
        "Hour of Day (UTC)", "Number of Telegrams Sent"
    ).set_title(
        &format!("Recruitment Activity by Hour of Day ({})", prettify_name(region))
    ).save(&output_path)?;

    Ok(fs::read(output_path)?)
}