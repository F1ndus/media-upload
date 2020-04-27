extern crate config;
extern crate lazy_static;
#[macro_use] extern crate failure;

use actix_web::{App, HttpServer, middleware, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::cfg::{parse_config, ServerConfig};
use crate::handler::*;
use crate::validation::*;

mod io;
mod metadata;
mod cfg;
mod validation;
mod handler;
mod exif_image;
mod exif_ffmpeg;

#[cfg(test)]
mod tests {
    //use crate::remove_metadata;
    use std::path::Path;

    #[test]
    fn it_works() {
       // remove_metadata(Path::new("/Users/findus/Downloads/exif.JPG"));
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    let config = parse_config();

    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    std::fs::create_dir_all(&config.path).unwrap();

    let ip = format!("{}:{}", &config.ip, &config.port);

    println!("Running at: {}", ip);
    println!("Path: {}", config.path);
    println!("URL: {}", config.url);

    HttpServer::new( move || {

        let auth = HttpAuthentication::bearer(validator);

        App::new()
            .data::<ServerConfig>(config.clone())
            .wrap(auth)
            .wrap(middleware::Logger::default())
            .service(
            web::resource("/")
                .route(web::post()
                    .to(save_file)),
        )
    })
        .bind(ip)?
        .run()
        .await
}