use std::{net::Ipv4Addr, process::ExitCode};

use anyhow::Result;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = dotenvy::dotenv() {
        eprintln!("Error with .env file: {err}");
        return ExitCode::FAILURE;
    }

    if let Err(err) = dictionary::run().await {
        eprintln!("{err:?}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
