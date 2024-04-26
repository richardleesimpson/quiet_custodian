use error::AppError;
use notify::{ Config, RecommendedWatcher, RecursiveMode, Watcher };
use std::{ fs::rename, path::PathBuf };

mod audio_utils;
mod cli;
mod error;
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
) -> Result<(), AppError> {
    std::fs::create_dir_all(&input).map_err(AppError::FileSystemError)?;

    let (handler, receiver) = std::sync::mpsc::channel();
    let mut event_watcher = RecommendedWatcher::new(handler, Config::default()).map_err(
        AppError::NotifyError
    )?;
    event_watcher.watch(&input, RecursiveMode::NonRecursive)?;

    for event in receiver {
        handle_event(
            event?,
            &output,
            replace_filename,
            append_timestamp,
            timestamp_format,
            append_summary
        )?;
    }

    Ok(())
}

fn handle_event(
    event: notify::Event,
    output: &PathBuf,
    replace_filename: bool,
    append_timestamp: bool,
    timestamp_format: &str,
    append_summary: bool
) -> Result<(), error::AppError> {
    debug!("Handling event: {:?}", event);
    for path in event.paths {
        if path.exists() && audio_utils::is_supported_audio_format(path.as_path()) {
            let to = output.join(
                file_ops::modify_filename(
                    &path,
                    replace_filename,
                    append_timestamp,
                    timestamp_format,
                    append_summary
                )?
            );
            audio_utils::transcribe_audio(&path, &to)?;
            file_ops::move_file(&path, &to)?;
        }
    }
    Ok(())
}
