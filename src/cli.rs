use std::path::PathBuf;
use clap::{ ArgAction, Parser };
use log::LevelFilter;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, help = "Input directory", default_value = "./")]
    pub input: PathBuf,
    #[arg(short, long, help = "Output directory", default_value = "./")]
    pub output: PathBuf,
    #[arg(
        short,
        long,
        help = "Replace filename. Cannot be used with --append-summary or --append-timestamp",
        action = ArgAction::SetTrue,
        conflicts_with_all = &["append_summary", "append_timestamp"]
    )]
    pub replace_filename: bool,
    #[arg(short = 's', long, help = "Append summary to filename", action = ArgAction::SetTrue)]
    pub append_summary: bool,
    #[arg(short = 't', long, help = "Append timestamp to filename", action = ArgAction::SetTrue)]
    pub append_timestamp: bool,
    #[arg(short = 'f', long, help = "Timestamp format", default_value = "%Y%m%d%H%M%S")]
    pub timestamp_format: String,
    #[arg(
        short,
        long,
        help = "The logging level (error, warn, info, debug, trace)",
        default_value = "info"
    )]
    pub log_level: LevelFilter,
}

pub fn parse() -> Args {
    Args::parse()
}
