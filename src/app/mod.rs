use crate::{repository, Config};
use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;

mod routes;

pub(crate) struct App {
    listener: TcpListener,
    router: Router,
}

impl App {
    pub(crate) async fn initialize(config: Config) -> Result<Self> {
        let listener = TcpListener::bind(config.address).await?;

        let shared_state = repository::Repository::initialize(config.database_url).await?;

        let router = routes::initialize_router(shared_state);

        Ok(Self { listener, router })
    }

    pub(crate) async fn run(self) -> Result<()> {
        axum::serve(self.listener, self.router).await?;

        Ok(())
    }
}
