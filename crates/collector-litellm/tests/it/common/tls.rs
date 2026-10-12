//! Throwaway certificates for the TLS tests, generated per run so no key material is committed.

use std::sync::Arc;

use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
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
    names.iter().map(|name| (*name).to_owned()).collect()
}

/// A fresh CA and a server certificate signed by it, valid for the given DNS names and IPs.
pub fn issued_by_new_ca(san: &[&str]) -> Pki {
    let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    ca_params.distinguished_name.push(DnType::CommonName, "litellm collector test CA");
    let ca_key = KeyPair::generate().unwrap();
    let ca_cert = ca_params.self_signed(&ca_key).unwrap();
    let issuer = Issuer::new(ca_params, ca_key);

    let mut leaf_params = CertificateParams::new(names(san)).unwrap();
    leaf_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    leaf_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let leaf_key = KeyPair::generate().unwrap();
    let leaf = leaf_params.signed_by(&leaf_key, &issuer).unwrap();
    Pki {
        ca_pem: ca_cert.pem(),
        server: server_config(vec![leaf.der().clone()], PrivateKeyDer::from(leaf_key)),
    }
}

/// A server certificate that signed itself: nothing but `Insecure` mode can trust it.
pub fn self_signed(san: &[&str]) -> Arc<ServerConfig> {
    let mut params = CertificateParams::new(names(san)).unwrap();
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let key = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key).unwrap();
    server_config(vec![cert.der().clone()], PrivateKeyDer::from(key))
}
