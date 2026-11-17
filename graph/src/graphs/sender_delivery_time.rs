use std::{error::Error, fs, iter::zip};
use itertools::Itertools;
use tempfile::TempDir;
use plotpy::{Barplot, Legend, Plot};
use vanille_models::report::ReportEntry;
use caramel::ns::format::prettify_name;

use crate::grouping::sender_delivery_time::group_data;
use crate::colors::DELIVERY_TIME_COLORS;
use crate::labels::INTERVAL_LABELS;

pub async fn generate_sender_delivery_time_plot(
    region: &str,
    entries: &[ReportEntry]
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let (values, x_labels) = group_data(entries);

    let tmp_dir = TempDir::new()?;
    let output_path = tmp_dir.path().join("graph.png");

    let mut bar = Barplot::new();
    let mut bottom = vec![0.0; x_labels.len()];
    let x_labels_ref = x_labels.iter().map(|s| s.as_str()).collect_vec();
    for (index, (label, col)) in zip(INTERVAL_LABELS, DELIVERY_TIME_COLORS).enumerate() {
        let vals = &values[index];
        bar.set_label(
            label
        ).set_colors(
            &[col]
        ).set_bottom(
            &bottom
        ).draw_with_str(
            &x_labels_ref, vals
        );

        for i in 0..x_labels.len() {
            bottom[i] += vals[i] as f64;
        }
    }

    let mut legend = Legend::new();
    legend.set_location("best");
    legend.draw();

    Plot::new().add(&bar).add(&legend).set_figure_size_inches(
        16f64, 9f64
    ).grid_labels_legend(
        "Sender", "Number of Telegrams Sent"
    ).set_title(
        &format!("Sender Leaderboard with Delivery Intervals ({})", prettify_name(region))
    ).set_rotation_ticks_x(45f64).extra(
        "plt.setp(plt.gca().get_xticklabels(), ha=\"right\")"
    ).set_save_tight(true).save(&output_path)?;

    Ok(fs::read(output_path)?)
}
