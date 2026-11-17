//! Réglages QUIC/TLS communs au client et au serveur.

use std::sync::Arc;

use rustls::crypto::CryptoProvider;

/// ALPN de HTTP/3 (RFC 9114), exigé par la spec v0.
pub const ALPN_H3: &[u8] = b"h3";

/// Le provider crypto de rustls : `ring`, le même que quinn.
pub fn crypto_provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}
