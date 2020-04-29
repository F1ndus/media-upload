use std::sync::RwLock;

use config::File;

#[derive(Clone, Debug, PartialEq)]
pub struct ServerConfig {
    pub path: String,
    pub token: String,
    pub url: String,
    pub port: u16,
    pub ip: String,
}

pub fn load_config<'a>(config: &'a mut config::Config) -> &'a  mut config::Config {
        config
        .merge(File::with_name("/etc/media-upload.toml"))
        .expect("Error loading config, make sure it is in /etc/media-upload.toml")
}

pub fn parse_config() -> ServerConfig {
    let mut default_config = config::Config::default();
    let cfg = load_config(&mut default_config);
    let rwlock = RwLock::new(cfg);
    let rw_lock = rwlock.write().expect("Error locking config file");

    crate::ServerConfig {
        path: rw_lock.get::<String>("path").unwrap(),
        token: rw_lock.get::<String>("token").unwrap(),
        url: rw_lock.get::<String>("url").unwrap(),
        port: rw_lock.get::<u16>("port").unwrap(),
        ip: rw_lock.get::<String>("ip").unwrap()
    }
}