#[path = "../src/plot_points_records.rs"]
mod plot_points_records;

use plot_points_records::{PlotPointsMode, append_t1_plot};
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 4 {
        return Err("Usage: plot_points_generate cartesian|polar BASE.gsp OUTPUT.gsp".into());
    }
    let mode = match args[1].as_str() {
        "cartesian" => PlotPointsMode::Cartesian,
        "polar" => PlotPointsMode::Polar,
        _ => return Err("Expected cartesian or polar".into()),
    };
    let base = fs::read(&args[2])?;
    let result = append_t1_plot(&base, mode, [1, 8])?;
    fs::write(&args[3], result)?;
    Ok(())
}
