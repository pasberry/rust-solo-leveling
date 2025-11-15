use clap::Parser;
use config_cli::{Config, process};
use std::path::Path;

#[derive(Parser)]
#[command(name = "dataproc")]
#[command(about = "Process data files based on configuration", long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load and validate configuration
    let config = Config::load(Path::new(&args.config))?;
    config.validate()?;

    println!("Configuration loaded successfully:");
    println!("  Input: {}", config.input.file);
    println!("  Operation: {}", config.processing.operation);
    println!("  Output: {}", config.output.file);
    println!();

    // Process data
    let result = process(&config)?;

    println!("Processing complete!");
    println!("  Processed {} items", result.count);
    println!("  Output written to {}", config.output.file);

    Ok(())
}
