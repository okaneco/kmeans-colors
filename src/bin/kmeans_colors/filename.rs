use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::err::CliError;

/// Creates a `PathBuf` to save the output filename. Handles the case where user
/// has specified an output and when there are multiple files that need names.
pub fn create_filename(
    input: &[PathBuf],
    output: &Option<PathBuf>,
    extension: &str,
    k: Option<u8>,
    file: &Path,
) -> Result<PathBuf, CliError> {
    let title;
    if input.len() == 1 {
        match output {
            Some(x) => {
                let mut temp = x.clone();
                match temp.extension() {
                    Some(_) => {}
                    None => {
                        temp.set_extension(extension);
                    }
                }
                title = temp;
            }
            None => {
                let mut temp = PathBuf::from(generate_filename(&file, k)?);
                temp.set_extension(extension);
                title = temp;
            }
        }
    } else {
        match output {
            Some(x) => {
                let mut temp = x.clone();
                let clone = temp.clone();
                let ext;
                match clone.extension() {
                    Some(y) => {
                        ext = y.to_str().unwrap();
                    }
                    None => {
                        ext = extension;
                    }
                }
                temp.set_file_name(format!(
                    "{}-{}",
                    &file.file_stem().unwrap().to_str().unwrap(),
                    &temp.file_stem().unwrap().to_str().unwrap()
                ));
                title = temp.with_extension(ext);
            }
            None => {
                let mut temp = PathBuf::from(generate_filename(&file, k)?);
                temp.set_extension(extension);
                title = temp;
            }
        }
    }

    Ok(title)
}

/// Creates a `PathBuf` to save the output palette.
pub fn create_filename_palette(
    input: &[PathBuf],
    output: &Option<PathBuf>,
    rgb: bool,
    k: Option<u8>,
    file: &Path,
) -> Result<PathBuf, CliError> {
    let title;
    let extension = "png";
    if input.len() == 1 {
        match output {
            Some(x) => {
                let mut temp = x.clone();
                match temp.extension() {
                    Some(_) => {}
                    None => {
                        temp.set_extension(extension);
                    }
                }
                title = temp;
            }
            None => {
                let mut temp = PathBuf::from(generate_filename_palette(&file, k.unwrap(), rgb)?);
                temp.set_extension(extension);
                title = temp;
            }
        }
    } else {
        match output {
            Some(x) => {
                let mut temp = x.clone();
                let clone = temp.clone();
                let ext;
                match clone.extension() {
                    Some(y) => {
                        ext = y.to_str().unwrap();
                    }
                    None => {
                        ext = extension;
                    }
                }
                temp.set_file_name(format!(
                    "{}-{}",
                    &file.file_stem().unwrap().to_str().unwrap(),
                    &temp.file_stem().unwrap().to_str().unwrap()
                ));
                title = temp.with_extension(ext);
            }
            None => {
                let mut temp = PathBuf::from(generate_filename_palette(&file, k.unwrap(), rgb)?);
                temp.set_extension(extension);
                title = temp;
            }
        }
    }

    Ok(title)
}

/// Appends a timestamp to an input filename to be used as output filename.
fn generate_filename(path: &Path, k: Option<u8>) -> Result<String, CliError> {
    let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let secs = now.as_secs();
    let millis = format!("{:03}", now.subsec_millis());
    match k {
        Some(x) => Ok(filename + "-" + &secs.to_string() + &millis + "-" + &x.to_string()),
        None => Ok(filename + "-" + &secs.to_string() + &millis),
    }
}

/// Appends a timestamp to an input filename to be used as a palette filename.
fn generate_filename_palette(path: &Path, k: u8, rgb: bool) -> Result<String, CliError> {
    let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let secs = now.as_secs();
    let millis = format!("{:03}", now.subsec_millis());
    let color = if rgb { "rgb" } else { "lab" };

    Ok(filename + "-" + &secs.to_string() + &millis + "-" + color + "-" + &k.to_string())
}
