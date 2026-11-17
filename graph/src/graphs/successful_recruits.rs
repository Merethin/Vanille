use std::{error::Error, fs, iter::zip};
use tempfile::TempDir;
use plotpy::{Barplot, Legend, Plot};
use vanille_models::report::ReportEntry;
use caramel::ns::format::prettify_name;

use crate::grouping::successful_recruits::group_data;
use crate::colors::CATEGORY_COLORS;
use crate::labels::{CATEGORY_LABELS, INTERVAL_LABELS};

pub async fn generate_successful_recruits_plot(
    region: &str,
    entries: &[ReportEntry]
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let values = group_data(entries);

    let tmp_dir = TempDir::new()?;
    let output_path = tmp_dir.path().join("graph.png");

    let mut bar = Barplot::new();
    let mut bottom = vec![0.0; INTERVAL_LABELS.len()];
    for (index, (label, col)) in zip(CATEGORY_LABELS, CATEGORY_COLORS).enumerate() {
        let vals = &values[index];
        bar.set_label(
            label
        ).set_colors(
            &[col]
        ).set_bottom(
            &bottom
        ).draw_with_str(
            &INTERVAL_LABELS, vals
        );

        for i in 0..INTERVAL_LABELS.len() {
            bottom[i] += vals[i] as f64;
        }
    }

    let mut legend = Legend::new();
    legend.set_location("best");
    legend.draw();

    Plot::new().add(&bar).add(&legend).set_figure_size_inches(
        16f64, 9f64
    ).grid_labels_legend(
        "Delivery Interval", "Number of nations that moved to the region"
    ).set_title(
        &format!("Successful Recruits by Delivery Interval ({})", prettify_name(region))
    ).set_save_tight(true).save(&output_path)?;

    Ok(fs::read(output_path)?)
}
