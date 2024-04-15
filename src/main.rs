use std::{ ffi::OsStr, fs::{ metadata, rename }, path::{ Path, PathBuf }, time::SystemTime };
use chrono::{ DateTime, Local, Utc };
use env_logger::Builder;
use notify::{ Config, RecommendedWatcher, RecursiveMode, Watcher };
use clap::{ ArgAction, Parser };
use log::{ debug, error, info, LevelFilter };

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, help = "Input directory", default_value = "./")]
    input: PathBuf,
    #[arg(short, long, help = "Output directory", default_value = "./")]
    output: PathBuf,
    #[arg(
        short,
        long,
        help = "Replace filename. Cannot be used with --append-summary or --append-timestamp",
        action = ArgAction::SetTrue,
        conflicts_with_all = &["append_summary", "append_timestamp"]
    )]
    replace_filename: bool,
    #[arg(short = 's', long, help = "Append summary to filename", action = ArgAction::SetTrue)]
    append_summary: bool,
    #[arg(short = 't', long, help = "Append timestamp to filename", action = ArgAction::SetTrue)]
    append_timestamp: bool,
    #[arg(short = 'f', long, help = "Timestamp format", default_value = "%Y%m%d%H%M%S")]
    timestamp_format: String,
    #[arg(
        short,
        long,
        help = "The logging level (error, warn, info, debug, trace)",
        default_value = "info"
    )]
    log_level: LevelFilter,
    // TODO: Make rename optional
}

fn main() {
    let cli: Cli = Cli::parse();
    Builder::from_default_env().filter(None, cli.log_level).init();

    info!("The custodian is quietly listening to {0:?} ...", cli.input);

    if
        let Err(error) = listen(
            cli.input,
            cli.output,
            cli.replace_filename,
            cli.append_timestamp,
            &cli.timestamp_format,
            cli.append_summary
        )
    {
        error!("Error listening: {error:?}");
    }
}

fn listen(
    input: PathBuf,
    output: PathBuf,
    replace_filename: bool,
    append_timestamp: bool,
    timestamp_format: &str,
    append_summary: bool
) -> notify::Result<()> {
    if let Err(error) = std::fs::create_dir_all(&input) {
        error!("Error creating source directory: {error:?}");
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let mut event_watcher = RecommendedWatcher::new(tx, Config::default())?;
    event_watcher.watch(&input, RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                debug!("Change: {event:?}");
                for path in event.paths {
                    if path.exists() && is_supported_audio_format(path.as_path()) {
                        let to = output.join(
                            reformat_filename(
                                &path,
                                replace_filename,
                                append_timestamp,
                                timestamp_format,
                                append_summary
                            )
                        );
                        if let Err(error) = transcribe_audio_file(&path, &to) {
                            error!("Error transcribing audio: {error:?}");
                        }
                        debug!("Moving {path:?} to {to:?}");
                        if let Err(error) = std::fs::create_dir_all(&output) {
                            error!("Error creating directory: {error:?}");
                        }
                        if let Err(error) = rename(path, to) {
                            error!("Error renaming: {error:?}");
                            continue;
                        }
                    }
                }
            }
            Err(error) => error!("Error: {error:?}"),
        }
    }

    Ok(())
}

fn reformat_filename(
    filename: &Path,
    replace_filename: bool,
    append_timestamp: bool,
    timestamp_format: &str,
    append_summary: bool
) -> String {
    let extension = filename
        .extension()
        .and_then(|s| s.to_str())
        .map(|ext| format!(".{}", ext))
        .unwrap_or_default();

    let default_stem = filename
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();

    let mut new_filename = if replace_filename { "".to_string() } else { default_stem.clone() };

    if append_timestamp || replace_filename {
        if let Some(date_time) = get_earliest_file_date(filename) {
            new_filename += &format!("_{}", date_time.format(timestamp_format));
        }
    }

    if append_summary || replace_filename {
        // TODO: Actual summary
        new_filename += "_summary";
    }

    new_filename + &extension
}

fn transcribe_audio_file(input: &Path, output: &Path) -> Result<(), String> {
    // TODO: Actual call to audio processing library
    debug!("Transcribing file from {:?} to {:?}", input, output);
    Ok(())
}

enum AudioFormat {
    MP3,
    MP4,
}

impl AudioFormat {
    fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(Self::MP3),
            "mp4" => Some(Self::MP4),
            _ => None,
        }
    }

    fn is_supported(ext: &str) -> bool {
        Self::from_extension(ext).is_some()
    }
}

fn is_supported_audio_format(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| AudioFormat::is_supported(ext))
        .unwrap_or(false)
}

fn get_earliest_file_date(path: &Path) -> Option<DateTime<Local>> {
    let meta = match metadata(path) {
        Ok(meta) => meta,
        Err(_) => {
            return None;
        }
    };

    let created_time = meta.created().unwrap_or(SystemTime::UNIX_EPOCH);
    let modified_time = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let earliest_time = if created_time < modified_time { created_time } else { modified_time };

    Some(DateTime::<Utc>::from(earliest_time).with_timezone(&Local))
}
