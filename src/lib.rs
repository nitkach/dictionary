use anyhow::Result;
use log::info;
use std::net::Ipv4Addr;
use url::Url;

mod app;
mod error;
mod model;
mod repository;

pub async fn run() -> Result<()> {
    let config = Config::new()?;
    info!("Config created");

    let app = app::App::initialize(config).await?;
    info!("App initialized");

    info!("Serving app");
    app.run().await?;

    Ok(())
}

pub(crate) struct Config {
    address: (Ipv4Addr, u16),
    database_url: Url,
}

impl Config {
    fn new() -> Result<Self> {
        let address = {
            let host = match std::env::var("HOST") {
                Ok(host) => host.parse::<Ipv4Addr>()?,
                Err(_) => Ipv4Addr::new(127, 0, 0, 1),
            };
            let port = match std::env::var("PORT") {
                Ok(port) => port.parse::<u16>()?,
                Err(_) => 3000,
            };
            (host, port)
        };

        let input = {
            let pg_user = std::env::var("PGUSER")?;
            let pg_password = std::env::var("PGPASSWORD")?;
            let pg_db = std::env::var("PGDATABASE")?;
            let pg_host = std::env::var("PGHOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
            let pg_port = std::env::var("PGPORT").unwrap_or_else(|_| "5432".to_owned());

            format!("postgres://{pg_user}:{pg_password}@postgres:{pg_port}/{pg_db}")
        };
        let database_url = Url::parse(&input)?;

        Ok(Self {
            address,
            database_url,
        })
    }
}
