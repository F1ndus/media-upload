#[macro_use]
extern crate lazy_static;
extern crate config;

use std::io::Write;

use actix_web_httpauth::middleware::HttpAuthentication;
use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::StreamExt;
use url::Url;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use actix_web::dev::{ServiceRequest, ResourcePath};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;

use std::sync::RwLock;
use config::{File};
use rexiv2::Rexiv2Error;

#[cfg(test)]
mod tests {
    use crate::remove_metadata;
    use std::path::Path;

    #[test]
    fn it_works() {
        remove_metadata(Path::new("/Users/findus/Downloads/exif.JPG"));
    }
}

lazy_static! {
	static ref SETTINGS: RwLock<config::Config> = RwLock::new(config::Config::default());
}

fn remove_metadata(filepath: &Path) -> Result<(), Rexiv2Error> {
    if filepath.exists() {
        println!("File exists");
    }
    let metadata = rexiv2::Metadata::new_from_path(filepath).unwrap();
    println!("{:#?}", metadata);
    metadata.clear_exif();
    metadata.save_to_file(filepath)?;
    Ok(())
}

fn copy_to_public(path: &Path, filename: &str) {
    let pub_path = SETTINGS.write().unwrap().get::<String>("path").unwrap();
    std::fs::copy(path, Path::new(&(pub_path + filename)));
}

async fn save_file(payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream

    println!("Save file");

    let mut filenames: Vec<String> = Vec::new();

    let p = &mut payload.enumerate();

    while let Some((i, item)) = p.next().await {
        let mut field = item?;
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let extension = Path::new(&filename).extension().expect("no extension?").to_os_string().to_str().expect("no ext?").to_owned();

        //let path = SETTINGS.read().get::<String>("path");
        let path = SETTINGS.write().unwrap().get::<String>("path").unwrap();

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let in_s = since_the_epoch.as_secs();

        let filepath = format!("{}/{}_{}.{}",path, in_s, i, extension);

        filenames.push(format!("{}_{}.{}", in_s, i, extension));

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }

    let url = SETTINGS.write().unwrap().get::<String>("url").unwrap();
    let links = filenames.iter()
        .map( |filename| url.to_owned() + filename)
        .map( |url| Url::parse(url.as_str()).expect("misdt").into_string() + "\n")
        .collect::<String>();

    Ok(HttpResponse::Ok().content_type("text/plain").body(links))
}

async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    println!("Validator");
     let token: String = SETTINGS.write().unwrap().get::<String>("token").unwrap();
     if credentials.token() == token {
         Ok(req)
     } else {
         let config = req.app_data::<Config>()
             .map(|data| data.get_ref().clone())
             .unwrap_or_else(Default::default)
             .scope("urn:example:channel=HBO&urn:example:rating=G,PG-13");

         Err(AuthenticationError::from(config).into())
     }
 }

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    SETTINGS.write().unwrap().merge(File::with_name("./config"));

    let p: String = SETTINGS.write().unwrap().get::<String>("path").unwrap();

    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    std::fs::create_dir_all(p).unwrap();

    let ip = "0.0.0.0:3000";

    println!("Running at: {}", ip);
    println!("Path: {}", SETTINGS.write().unwrap().get::<String>("path").unwrap());
    println!("URL: {}", SETTINGS.write().unwrap().get::<String>("url").unwrap());

    HttpServer::new(|| {

        let auth = HttpAuthentication::bearer(validator);

        App::new()
            .wrap(auth)
            .wrap(middleware::Logger::default())
            .service(
            web::resource("/")
                .route(web::post().to(save_file)),
        )
    })
        .bind(ip)?
        .run()
        .await
}