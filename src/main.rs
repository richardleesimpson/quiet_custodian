use std::{ fs::rename, path::PathBuf };
use notify::{ Config, RecommendedWatcher, RecursiveMode, Watcher };
mod audio_utils;
mod cli;
mod file_ops;
mod logging;

fn main() {
    let cli: cli::Args = cli::parse();
    logging::init(cli.log_level);

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
                    if path.exists() && audio_utils::is_supported_audio_format(path.as_path()) {
                        let to = output.join(
                            file_ops::modify_filename(
                                &path,
                                replace_filename,
                                append_timestamp,
                                timestamp_format,
                                append_summary
                            )
                        );
                        if let Err(error) = audio_utils::transcribe_audio(&path, &to) {
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
