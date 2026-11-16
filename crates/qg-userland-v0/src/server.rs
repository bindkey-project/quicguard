//! Côté serveur : accepte les clients et leur attribue une adresse.
use anyhow::Result;

use crate::cli::ServerArgs;

pub async fn run(args: ServerArgs) -> Result<()> {
    tracing::info!(listen = %args.listen, pool=%args.pool, "démarrage du serveur ");
    tracing::debug!(?args, "arguments du serveur");
    Ok(())
}
