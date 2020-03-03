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


lazy_static! {
	static ref SETTINGS: RwLock<config::Config> = RwLock::new(config::Config::default());
}

async fn save_file(payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream

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