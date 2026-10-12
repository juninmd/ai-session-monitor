//! Throwaway certificates for TLS tests, generated per run so no key material is committed.

use std::sync::Arc;

use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose, date_time_ymd,
};
use rustls::ServerConfig;
use rustls::crypto::aws_lc_rs::default_provider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

pub struct Pki {
    /// The CA certificate a client must trust to accept the server.
    pub ca_pem: String,
    pub server: Arc<ServerConfig>,
}

fn server_config(
    chain: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> Arc<ServerConfig> {
    let config = ServerConfig::builder_with_provider(Arc::new(default_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(chain, key)
        .unwrap();
    Arc::new(config)
}

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

fn build(san: &[&str], expired: bool) -> Pki {
    let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    ca_params.distinguished_name.push(DnType::CommonName, "usage-support test CA");
    let ca_key = KeyPair::generate().unwrap();
    let ca_cert = ca_params.self_signed(&ca_key).unwrap();
    let issuer = Issuer::new(ca_params, ca_key);

    let mut leaf_params = CertificateParams::new(names(san)).unwrap();
    leaf_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    leaf_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    if expired {
        leaf_params.not_before = date_time_ymd(2001, 1, 1);
        leaf_params.not_after = date_time_ymd(2002, 1, 1);
    }
    let leaf_key = KeyPair::generate().unwrap();
    let leaf = leaf_params.signed_by(&leaf_key, &issuer).unwrap();
    Pki {
        ca_pem: ca_cert.pem(),
        server: server_config(vec![leaf.der().clone()], PrivateKeyDer::from(leaf_key)),
    }
}

/// A fresh CA and a server certificate signed by it, valid for the given DNS names and IPs.
pub fn issued_by_new_ca(san: &[&str]) -> Pki {
    build(san, false)
}

/// Like [`issued_by_new_ca`], but the server certificate ran out in 2002.
pub fn expired_by_new_ca(san: &[&str]) -> Pki {
    build(san, true)
}

/// A server certificate that signed itself: nothing but `Insecure` mode can trust it.
pub fn self_signed(san: &[&str]) -> Arc<ServerConfig> {
    let mut params = CertificateParams::new(names(san)).unwrap();
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let key = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key).unwrap();
    server_config(vec![cert.der().clone()], PrivateKeyDer::from(key))
}
