use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use actix_multipart::{Field};
use actix_web::http::header::{ContentDisposition};
use futures::StreamExt;
use failure::Error;
use actix_web::web;
use std::fs::File;

pub(crate) fn copy_file(file: &PathBuf, destination: &Path) -> Result<(), Error> {
    println!("Move img to {}", destination.to_str().unwrap().to_owned());
    std::fs::copy(file, &destination)?;
    Ok(())
}

pub(crate) fn get_filename_extension(path: &Path) -> Option<String> {
        match path.extension()?.to_os_string().to_str() {
            Some(str) =>  Some(str.to_string()),
            None => None,
        }
}

#[derive(Debug, Fail)]
enum IOError {
    #[fail(display = "Could not extract Filename")]
    CouldNotExtractFilename,
    #[fail(display = "Could not get filename extension")]
    CouldNotGetFileNameExtension,
}

pub(crate) fn generate_public_filename(content_type: &ContentDisposition, iteration: usize)
    -> Result<String, Error>
{

    let filename = content_type.get_filename()
        .ok_or(IOError::CouldNotExtractFilename)?;

    let extension = get_filename_extension(Path::new(&filename))
        .ok_or(IOError::CouldNotGetFileNameExtension)?;

    let start = SystemTime::now();

    let since_the_epoch = start.duration_since(UNIX_EPOCH)?;

    let epoch_in_seconds = since_the_epoch.as_secs();

    let filename_pub_epoch = format!("{}_{}.{}", epoch_in_seconds, iteration, extension);

    Ok(format!("{}", filename_pub_epoch))
}

pub(crate) async fn save_file_to_temp_folder(field: &mut Field, temp_path: String) -> Result<(), actix_web::Error> {
    let mut f = web::block(|| std::fs::File::create(temp_path)).await.unwrap();
    // Field in turn is stream of *Bytes* object
    while let Some(chunk) = field.next().await {
        let data = chunk.unwrap();
        // filesystem operations are blocking, we have to use threadpool
        f = web::block(move || f.write_all(&data).map(|_| f)).await?;
    }
    Ok(())
}