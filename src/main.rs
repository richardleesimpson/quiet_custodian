#[macro_use]
extern crate lazy_static;

const INPUT_DATE_FORMAT: &str = "%y%m%d%H%M";
const OUTPUT_DATE_FORMAT: &str = "%Y-%m-%d %H-%M";

use std::{ env, fs::rename, path::PathBuf };
use chrono::NaiveDateTime;
use notify::{ Config, RecommendedWatcher, RecursiveMode, Watcher };
use regex::Regex;

lazy_static! {
    static ref FILENAME_PATTERN: Regex = Regex::new(
        r"R-\d{5}_(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})_REC(?i)\.mp3$"
    ).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <source_folder> <destination_folder>", args[0]);
        std::process::exit(1);
    }

    let source_folder = PathBuf::from(&args[1]);
    let destination_folder = PathBuf::from(&args[2]);

    println!("The custodian is quietly listening to {source_folder:?} ...");

    if let Err(error) = watch(source_folder, destination_folder) {
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
                        rename(path, to).unwrap();
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
