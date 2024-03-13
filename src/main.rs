use std::{env, path::PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source_folder>", args[0]);
        std::process::exit(1);
    }
    
    let source_folder = PathBuf::from(&args[1]);

    println!("{source_folder:?}");
}
