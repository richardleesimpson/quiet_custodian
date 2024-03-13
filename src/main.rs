use std::{ env, path::{ Path, PathBuf } };
use notify::{ Config, RecommendedWatcher, RecursiveMode, Watcher };

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source_folder>", args[0]);
        std::process::exit(1);
    }

    let source_folder = PathBuf::from(&args[1]);

    println!("The custodian is quietly listening to {source_folder:?} ...");

    if let Err(error) = watch(source_folder) {
        eprintln!("Error: {error:?}");
    }
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => println!("Change: {event:?}"),
            Err(error) => eprintln!("Error: {error:?}"),
        }
    }

    Ok(())
}
