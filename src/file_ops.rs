use std::{ fs::metadata, path::Path, time::SystemTime };
use chrono::{ DateTime, Local, Utc };

pub fn modify_filename(
    path: &Path,
    replace_filename: bool,
    append_timestamp: bool,
    timestamp_format: &str,
    append_summary: bool
) -> String {
    let extension = get_extension(path);
    let mut filename = if replace_filename { "".to_string() } else { get_filename(path) };

    if append_timestamp || replace_filename {
        filename = add_timestamp(path, &filename, timestamp_format);
    }

    if append_summary || replace_filename {
        filename = add_summary(&filename);
    }

    filename + &extension
}

fn add_summary(filename: &str) -> String {
    // TODO: Actual summary
    format!("{}_summary", filename)
}

fn add_timestamp(path: &Path, filename: &str, timestamp_format: &str) -> String {
    if let Some(timestamp) = get_timestamp(path, timestamp_format) {
        format!("{}_{}", filename, timestamp)
    } else {
        filename.to_string()
    }
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

fn get_extension(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| format!(".{}", ext))
        .unwrap_or_default()
}

fn get_filename(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string()
}

fn get_timestamp(path: &Path, format: &str) -> Option<String> {
    get_earliest_file_date(path).map(|date_time| date_time.format(format).to_string())
}
