
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::io::{self, Write};
use failure::Error;

pub(crate) fn remove_video_metadata(filepath: &Path) -> Option<PathBuf> {

    // Check if file exists
    match filepath.exists() {
        true => {
            if let Ok(path) = convert_file(filepath) {
                Some(path)
            } else {
                None
            }
        }
        _ => {
            None
        }
    }

}

#[derive(Debug, Fail)]
enum ConvertError {
    #[fail(display = "File Creation Failed Error")]
    FileCreationFailed,
    #[fail(display = "ToStr Failed")]
    toStrFailed,
    #[fail(display = "FFmpegConversion Failed")]
    FFmpegConversionFailed,
}

fn convert_file(file: &Path) -> Result<PathBuf, Error> {

    // let extension = file.extension()?.to_str()?;
    let filename = file.file_name()
        .ok_or(ConvertError::FileCreationFailed)?
        .to_str().ok_or(ConvertError::toStrFailed)?;

    let out = format!("/tmp/processed/{}", filename);
    std::fs::create_dir_all(Path::new("/tmp/processed/"))?;
    let output = Command::new("ffmpeg")
        .arg("-y")
        .args(&["-i", file.to_str().ok_or(ConvertError::toStrFailed)?])
        .args(&["-map_metadata", "-1"])
        .args(&["-c:v","copy"])
        .args(&["-c:a","copy"])
        .arg(&out)
        .output()?;

    println!("status: {}", output.status);

    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;

    println!("{}", output.status.success());
    if output.status.code() != Some(0) {
         bail!("FFmpeg quit with a non zero exit code!");
    }

    let mut path = PathBuf::new();
    path.push(out);

    Ok(path)
}

#[cfg(test)]
mod test_ffmpeg {
    //use crate::remove_metadata;
    use std::path::Path;
    use crate::exif_ffmeg::convert_file;

    #[test]
    fn it_works() {
        convert_file(Path::new("/Users/findus/Videos/Zeitraffer.mp4"));
    }
}