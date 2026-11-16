//! QuicGuard v0 : VPN MASQUE CONNECT-IP en espace utilisateur, construit sur quinn.
mod cert;
mod cli;
mod client;
mod server;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Logs : niveau choisi par RUST_LOG, "info" par défaut.
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // 2. Lecture de la ligne de commande (quitte avec un message clair si elle est invalide).
    let cli = Cli::parse();

    // 3. Aiguillage vers le mode demandé.
    match cli.command {
        Command::Server(args) => server::run(args).await,
        Command::Client(args) => client::run(args).await,
        Command::GenCert(args)=> cert::generate(args),
        
    }
}
