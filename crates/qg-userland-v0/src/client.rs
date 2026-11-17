//! Côté client : se connecte au serveur et monte le tunnel.

use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::{Context, Ok, Result};
use quinn::crypto::rustls::QuicClientConfig;
use rustls::pki_types::CertificateDer;

use crate::cli::ClientArgs;
use crate::quic;

pub async fn run(args: ClientArgs) -> Result<()> {
    let config = client_config(&args)?;

    // Port local 0 : le système choisit un port UDP libre.
    let mut endpoint = quinn::Endpoint::client(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))?;
    endpoint.set_default_client_config(config);

    tracing::info!(server = %&args.server, name = &args.name, "connexion au serveur");
    let connecting = endpoint.connect(args.server, &args.name)?;
    let conn = connecting
        .await
        .with_context(|| format!("échec de la connexion QUIC à {}", args.server))?;

    tracing::info!(
        rtt= ?conn.rtt(),
        max_datagram_size = ?&conn.max_datagram_size(),
        "connexion QUIC établie"
    );

    // Fermeture propre : code applicatif 0 = "pas d'erreur".
    conn.close(quinn::VarInt::from_u32(0), b"bye");
    endpoint.wait_idle().await;
    Ok(())
}

/// Construit la configuration TLS 1.3 + QUIC du client.
fn client_config(args: &ClientArgs) -> Result<quinn::ClientConfig> {
    let ca = fs::read(&args.ca)
        .with_context(|| format!("Lecture du certificat CA {}", args.ca.display()))?;

    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from(ca))
        .context("certificat CA invalide")?;

    let mut tls = rustls::ClientConfig::builder_with_provider(quic::crypto_provider())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls.alpn_protocols = vec![quic::ALPN_H3.to_vec()];
    tls.key_log = Arc::new(rustls::KeyLogFile::new());

    let tls = QuicClientConfig::try_from(tls)?;
    Ok(quinn::ClientConfig::new(Arc::new(tls)))
}
