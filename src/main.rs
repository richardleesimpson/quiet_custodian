use std::{ env, fs::{ rename }, path::PathBuf };

use notify::{ Config, RecommendedWatcher, RecursiveMode, Watcher };

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
                    if path.exists() {
                        let to = destination.join(path.file_name().unwrap());
                        println!("Moving {path:?} to {to:?}");
                        rename(path, to).unwrap();
                    }
                }
            }
            Err(error) => println!("Error: {error:?}"),
        }
    }

    Ok(())
}
