use std::{ ffi::OsStr, fs::{ metadata, rename }, path::{ Path, PathBuf } };
use chrono::{ DateTime, Utc };
use notify::{ Config, RecommendedWatcher, RecursiveMode, Watcher };
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "INPUT_DIR", help = "Input directory", default_value = "./")]
    input: PathBuf,
    #[arg(short, long, value_name = "OUTPUT_DIR", help = "Output directory", default_value = "./")]
    output: PathBuf,
    #[arg(
        short = 'f',
        long,
        value_name = "OUTPUT_FORMAT",
        help = "Output format",
        default_value = "%Y-%m-%d %H-%M"
    )]
    output_format: String, // TODO: Rename this
}

fn main() {
    let cli: Cli = Cli::parse();

    println!("The custodian is quietly listening to {0:?} ...", cli.input);

    if let Err(error) = listen(cli.input, cli.output, &cli.output_format) {
        eprintln!("Error listening: {error:?}");
    }
}

fn listen(input: PathBuf, output: PathBuf, output_format: &str) -> notify::Result<()> {
    if let Err(error) = std::fs::create_dir_all(&input) {
        eprintln!("Error creating source directory: {error:?}");
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let mut event_watcher = RecommendedWatcher::new(tx, Config::default())?;
    event_watcher.watch(&input, RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                println!("Change: {event:?}");
                for path in event.paths {
                    if path.exists() && is_supported_audio_format(path.as_path()) {
                        let to = output.join(reformat_filename(&path, output_format));
                        if let Err(error) = transcribe_audio_file(&path, &to) {
                            eprintln!("Error transcribing audio: {error:?}");
                        }
                        println!("Moving {path:?} to {to:?}");
                        if let Err(error) = std::fs::create_dir_all(&output) {
                            eprintln!("Error creating directory: {error:?}");
                        }
                        if let Err(error) = rename(path, to) {
                            eprintln!("Error renaming: {error:?}");
                            continue;
                        }
                    }
                }
            }
            Err(error) => println!("Error: {error:?}"),
        }
    }

    Ok(())
}

fn reformat_filename(filename: &Path, output_format: &str) -> String {
    if let Some(date_time) = get_file_date(filename) {
        format!("{}.mp3", date_time.format(output_format))
    } else {
        filename.to_str().unwrap().to_string()
    }
}

fn transcribe_audio_file(input: &Path, output: &Path) -> Result<(), String> {
    // TODO: Actual call to audio processing library
    println!("Transcribing file from {:?} to {:?}", input, output);
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

fn get_file_date(path: &Path) -> Option<DateTime<Utc>> {
    let meta = match metadata(path) {
        Ok(meta) => meta,
        Err(_) => {
            return None;
        }
    };

    let creation_date = meta.created().or_else(|_| meta.modified());
    match creation_date {
        Ok(system_time) => Some(DateTime::<Utc>::from(system_time)),
        Err(_) => None,
    }
}
