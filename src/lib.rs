use anyhow::Result;
use std::net::Ipv4Addr;
use url::Url;

mod app;
mod dto;
mod error;
mod model;
mod repository;

pub async fn run() -> Result<()> {
    let config = Config::new()?;

    let app = app::App::initialize(config).await?;

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
            let pg_user = std::env::var("PG_USER")?;
            let pg_password = std::env::var("PG_PASSWORD")?;
            let pg_db = std::env::var("PG_DB")?;

            format!("postgres://{pg_user}:{pg_password}@localhost:5432/{pg_db}")
        };
        let database_url = Url::parse(&input)?;

        Ok(Self {
            address,
            database_url,
        })
    }
}
