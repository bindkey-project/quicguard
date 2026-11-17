//! Génération d'un certificat auto-signé pour le labo (pas pour la production).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

use anyhow::{Context, Result};

use crate::cli::GenCertArgs;

pub fn generate(args: GenCertArgs) -> Result<()> {
    let certified = rcgen::generate_simple_self_signed(vec![args.name.clone()])
        .context("generation du certificat")?;

    fs::write(&args.cert, certified.cert.der())
        .with_context(|| format!("écriture de {}", args.cert.display()))?;

    // La clé privée n'est lisible que par son propriétaire (rw-------).
    let mut key_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&args.key)
        .with_context(|| format!("écriture de {}", args.key.display()))?;
    key_file
        .write_all(&certified.key_pair.serialize_der())
        .with_context(|| format!("écriture de {}", args.key.display()))?;

    tracing::info!(
        name = %args.name,
        cert = %args.cert.display(),
        key = %args.key.display(),
            "certificat auto-signé généré"
    );
    Ok(())
}
