#![allow(dead_code)]

use std::sync::OnceLock;

pub fn install_rustls_crypto_provider() {
    static INSTALLED: OnceLock<()> = OnceLock::new();
    INSTALLED.get_or_init(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}
