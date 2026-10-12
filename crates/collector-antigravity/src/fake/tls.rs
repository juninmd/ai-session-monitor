//! A throwaway self-signed certificate for the HTTPS stand-in: new key on every run, nothing
//! committed, valid for `127.0.0.1` only. The collector accepts it only because it asked for
//! `TlsPolicy::Insecure` on that one host.

use std::sync::Arc;

use rcgen::{CertificateParams, ExtendedKeyUsagePurpose, KeyPair};
use rustls::ServerConfig;
use rustls::crypto::aws_lc_rs::default_provider;
use rustls::pki_types::PrivateKeyDer;
use tokio_rustls::TlsAcceptor;

pub(super) fn acceptor() -> TlsAcceptor {
    let mut params = CertificateParams::new(vec!["127.0.0.1".to_owned()]).unwrap();
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let key = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key).unwrap();
    let config = ServerConfig::builder_with_provider(Arc::new(default_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert.der().clone()], PrivateKeyDer::from(key))
        .unwrap();
    TlsAcceptor::from(Arc::new(config))
}
