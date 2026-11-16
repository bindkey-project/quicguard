//! Côté client : se connecte au serveur et monte le tunnel.

use anyhow::Result;

use crate::cli::ClientArgs;

pub async fn run(args: ClientArgs) -> Result<()> {
    tracing::info!(server=%args.server, name= %args.name, "démarrage du client");
    tracing::debug!(?args, "arguments du client");
    Ok(())
}
