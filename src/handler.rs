use std::path::Path;

use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse};
use actix_web::web::Data;
use futures::StreamExt;
use url::Url;

use crate::cfg::ServerConfig;
use crate::io::*;
use crate::metadata;

pub(crate) async fn save_file(payload: Multipart, data: Data<ServerConfig>) -> Result<HttpResponse, Error> {
    // iterate over multipart stream

    println!("Save file");

    let config = data.get_ref().clone();

    let mut filenames: Vec<String> = Vec::new();

    let p = &mut payload.enumerate();

    while let Some((i, item)) = p.next().await {
        let mut field = item?;
        let content_type = field.content_disposition().unwrap();

        let public_filename =
            generate_public_filename(content_type, i)
            .expect("Cannot generate public filename");

        filenames.push(format!("{}", public_filename));

        let temp_path = format!("/tmp/{}", public_filename);

        save_file_to_temp_folder(&mut field, temp_path.clone()).await?;

        let extension = get_filename_extension(Path::new(&public_filename))
            .expect("Cannot get filename extension");

        let path = format!("{}/{}", config.path, public_filename);
        let public_path= Path::new(&path);

        match extension.to_lowercase().as_str() {
            "mp4" |  "mov" | "webm" => {
                //Err(ErrorInternalServerError("Blacklisted filetype"))
                continue
            },
            "jpg" | "png" => {
                match metadata::remove_img_metadata(temp_path.as_ref()) {
                    Ok(_) => {
                        copy_file(temp_path.as_ref(), Path::new(public_path));
                    }
                    _ => {
                        continue
                    }
                }

            },
            _ => {
                copy_file(temp_path.as_ref(), public_path);
            }
        }


    }

    let links = filenames.iter()
        .map( |filename| format!("{}{}", config.url.to_string(), filename))
        .collect::<String>();

    Ok(HttpResponse::Ok().content_type("text/plain").body(links))
}