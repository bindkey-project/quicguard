//! Côté serveur : accepte les clients et leur attribue une adresse.

use std::fs;
use std::sync::Arc;

use anyhow::{Context, Result};
use quinn::crypto::rustls::QuicServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

use crate::cli::ServerArgs;
use crate::quic;

pub async fn run(args: ServerArgs) -> Result<()> {
    let config = server_config(&args)?;
    let endpoint = quinn::Endpoint::server(config, args.listen)
        .with_context(|| format!("écoute impossible sur {}", args.listen))?;

    tracing::info!(listen = %endpoint.local_addr()?, pool = %args.pool, "serveur en écoute");

    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_connection(incoming).await {
                tracing::warn!(error = %e, "connexion échouée");
            }
        });
    }
    Ok(())
}

/// Construit la configuration TLS 1.3 + QUIC du serveur.
fn server_config(args: &ServerArgs) -> Result<quinn::ServerConfig> {
    let cert = fs::read(&args.cert)
        .with_context(|| format!("lecture du certificat {}", args.cert.display()))?;
    let key =
        fs::read(&args.key).with_context(|| format!("lecture de la clé {}", args.key.display()))?;

    let cert = CertificateDer::from(cert);
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key));

    let mut tls = rustls::ServerConfig::builder_with_provider(quic::crypto_provider())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .context("certificat ou clé invalide")?;
    tls.alpn_protocols = vec![quic::ALPN_H3.to_vec()];
    tls.key_log = Arc::new(rustls::KeyLogFile::new());

    let tls = QuicServerConfig::try_from(tls)?;
    Ok(quinn::ServerConfig::with_crypto(Arc::new(tls)))
}

/// Termine le handshake d'un client puis attend la fin de la connexion.
async fn handle_connection(incoming: quinn::Incoming) -> Result<()> {
    let remote = incoming.remote_address();
    let conn = incoming.await?;
    tracing::info!(%remote, "client connecté");

    let reason = conn.closed().await;
    tracing::info!(%remote, %reason, "client déconnecté");
    Ok(())
}
