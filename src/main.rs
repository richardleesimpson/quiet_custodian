#[macro_use]
extern crate lazy_static;

const INPUT_DATE_FORMAT: &str = "%y%m%d%H%M";
const OUTPUT_DATE_FORMAT: &str = "%Y-%m-%d %H-%M";

use std::{ fs::rename, path::PathBuf };
use chrono::NaiveDateTime;
use notify::{ Config, RecommendedWatcher, RecursiveMode, Watcher };
use regex::Regex;
use clap::Parser;

lazy_static! {
    static ref FILENAME_PATTERN: Regex = Regex::new(
        r"R-\d{5}_(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})_REC(?i)\.mp3$"
    ).unwrap();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "INPUT_DIR", help = "Input directory", default_value = "./")]
    input: PathBuf,
    #[arg(short, long, value_name = "OUTPUT_DIR", help = "Output directory", default_value = "./")]
    output: PathBuf,
}

fn main() {
    let cli: Cli = Cli::parse();

    println!("The custodian is quietly listening to {0:?} ...", cli.input);

    if let Err(error) = watch(cli.input, cli.output) {
        eprintln!("Error: {error:?}");
    }
}

fn watch(source: PathBuf, destination: PathBuf) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(&source, RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                println!("Change: {event:?}");
                for path in event.paths {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let is_match = FILENAME_PATTERN.is_match(file_name);
                    if path.exists() && is_match {
                        let to = destination.join(reformat_filename(file_name));
                        println!("Moving {path:?} to {to:?}");
                        if let Err(error) = std::fs::create_dir_all(&destination) {
                            eprintln!("Error: {error:?}");
                        }
                        if let Err(error) = rename(path, to) {
                            eprintln!("Error: {error:?}");
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

fn reformat_filename(filename: &str) -> String {
    println!("Reformatting filename: {filename}");
    if let Some(caps) = FILENAME_PATTERN.captures(filename) {
        let date_str = format!("{}{}{}{}{}", &caps[1], &caps[2], &caps[3], &caps[4], &caps[5]);

        if let Ok(date_time) = NaiveDateTime::parse_from_str(&date_str, INPUT_DATE_FORMAT) {
            format!("{}.mp3", date_time.format(OUTPUT_DATE_FORMAT))
        } else {
            eprintln!("Could not parse date from filename: {}", filename);
            filename.to_string()
        }
    } else {
        filename.to_string()
    }
}
