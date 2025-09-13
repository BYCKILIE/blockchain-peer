use std::path::Path;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::pki_types::pem::PemObject;
use rustls::{ClientConfig, RootCertStore};
use common::config::nats_config::NatsConfig;

pub fn build_tls_config(nats_config: &NatsConfig) -> ClientConfig {
    let mut root_cert_store = RootCertStore::empty();
    for cert in load_certs(&format!("{}/chain.pem", &nats_config.cert_store_path)) {
        root_cert_store.add(cert).expect("Failed to add cert to root");
    }

    ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(
            load_certs(&format!("{}/fullchain.pem", &nats_config.cert_store_path)),
            load_key(&format!("{}/privkey.pem",   &nats_config.cert_store_path)),
        )
        .expect("Failed to build client config")
}

fn load_certs(path: &str) -> Vec<CertificateDer<'static>> {
    let pem = std::fs::read(path).expect("cannot read cert file");
    let mut reader: &[u8] = &pem;

    rustls_pemfile::certs(&mut reader)
        .filter_map(Result::ok)
        .map(CertificateDer::from)
        .collect()
}

fn load_key(
    path: impl AsRef<Path>,
) -> PrivateKeyDer<'static> {
    let key = PrivateKeyDer::from_pem_file(&path).expect("Failed to load private key");
    key.clone_key()
}