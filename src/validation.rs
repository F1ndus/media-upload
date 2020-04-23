use actix_web::dev::{ServiceRequest};
use actix_web::Error;
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};

use crate::cfg::ServerConfig;
use actix_web::web::Data;

pub(crate) async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    let config = req.app_data::<ServerConfig>();
    let config = config.unwrap();
    let token: String = (&config.token).parse().unwrap();
    if credentials.token() == token {
        Ok(req)
    } else {
        let config = req.app_data::<Config>()
            .map(|data| data.get_ref().clone())
            .unwrap_or_else(Default::default);
        Err(AuthenticationError::from(config).into())
    }
}