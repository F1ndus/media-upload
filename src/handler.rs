use std::path::Path;

use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse};
use actix_web::web::Data;
use futures::StreamExt;
use url::Url;

use crate::cfg::ServerConfig;
use crate::io::*;
use crate::metadata;
use std::fs::metadata;
use actix_web::error::ErrorInternalServerError;
use infer::Infer;
use infer::Type;

pub(crate) async fn save_file(payload: Multipart, data: Data<ServerConfig>) -> Result<HttpResponse, Error> {
    // iterate over multipart stream

    println!("Save file");

    let config = data.get_ref().clone();

    let mut filenames: Vec<String> = Vec::new();

    let p = &mut payload.enumerate();

    while let Some((i, item)) = p.next().await {
        let mut field = item?;

        let content_type = field.content_disposition().unwrap();
        let user_filename = content_type.get_filename().unwrap();

        let public_filename =
            generate_public_filename(&content_type, i)
            .expect("Cannot generate public filename");

        filenames.push(format!("{}", public_filename));

        let temp_path = format!("/tmp/{}", public_filename);

        save_file_to_temp_folder(&mut field, temp_path.clone()).await?;

        let extension = get_filename_extension(Path::new(&public_filename))
            .expect("Cannot get filename extension");

        let path = format!("{}/{}", config.path, public_filename);
        let public_path= Path::new(&path);

        let file_type_analyzer = infer::Infer::new();
        let filetype = file_type_analyzer.get_from_path(&temp_path).unwrap();

        let meta_data: Option<Box<dyn metadata::MetaData>> = match filetype {
            Some(t) if t.mime.contains("image")
                => Some(Box::new(metadata::Image { path: temp_path.as_ref() })),

            Some(t) if t.mime.contains("video")
                => Some(Box::new(metadata::VideoFile { path: temp_path.as_ref()})),
            Some(_) => Some(Box::new(metadata::Noop { path: temp_path.as_ref() })),
            None => None
        };

        if let Some(meta_data) = meta_data {

            if let Ok(stripped_file_path) = meta_data.as_ref().remove_metadata() {
                copy_file(&stripped_file_path, Path::new(public_path));
            } else {
                println!("Error Occured while removing metadata");
                return Err(ErrorInternalServerError(
                    format!("Something went wrong while stripping the metadata of {}",
                            user_filename)));
            }
        } else {
            println!("No handler found for this filetype");
            return Err(ErrorInternalServerError(
                format!("Sorry, I cannot handle this sort of file {}", user_filename)
            ))
        }

    }

    let links = filenames.iter()
        .map( |filename| format!("{}{}\n", config.url.to_string(), filename))
        .collect::<String>();

    Ok(HttpResponse::Ok().content_type("text/plain").body(links))
}