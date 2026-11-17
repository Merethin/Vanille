use std::{error::Error, fs, iter::zip};
use itertools::Itertools;
use tempfile::TempDir;
use plotpy::{Barplot, Legend, Plot};
use vanille_models::report::ReportEntry;
use caramel::ns::format::prettify_name;
use random_color::{RandomColor, options::Luminosity};

use crate::grouping::sender_activity::group_data;

pub async fn generate_sender_activity_plot(
    region: &str,
    entries: &[ReportEntry]
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let (values, labels) = group_data(entries);
    let mut generator = RandomColor::new();
    generator.luminosity(Luminosity::Bright);

    let colors = labels.iter().map(|_| generator.to_hex()).collect_vec();

    let tmp_dir = TempDir::new()?;
    let output_path = tmp_dir.path().join("graph.png");

    let mut bar = Barplot::new();
    let mut bottom = vec![0.0; 24];
    let x_labels: Vec<_> = (0..24).map(|hour| format!("{:02}:00", hour)).collect();
    let x_labels_ref = x_labels.iter().map(|s| s.as_str()).collect_vec();
    for (index, (label, col)) in zip(&labels, &colors).enumerate() {
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

        for i in 0..24 {
            bottom[i] += vals[i] as f64;
        }
    }

    let mut legend = Legend::new();
    legend.set_location("best");
    legend.draw();

    Plot::new().add(&bar).add(&legend).set_figure_size_inches(
        16f64, 9f64
    ).grid_labels_legend(
        "Hour of Day (UTC)", "Number of Telegrams Sent"
    ).set_title(
        &format!("Sender Activity by Hour of Day ({})", prettify_name(region))
    ).set_save_tight(true).save(&output_path)?;

    Ok(fs::read(output_path)?)
}
