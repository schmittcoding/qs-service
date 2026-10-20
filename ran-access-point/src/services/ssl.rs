use rustls::{Certificate, PrivateKey, RootCertStore, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::env;
use std::io::{BufReader, Cursor};
use tokio::fs;
use tracing::info;

/// Load Cloudflare Origin CA certificate into a RootCertStore.
/// Typically used for client configuration. For a server,
/// you use your own certificate and key.
pub async fn load_cf_origin_ca() -> RootCertStore {
    dotenvy::dotenv().ok();
    let mut root_store = RootCertStore::empty();
    let root_path = env::var("CERT_PATH").expect("CERT_PATH must be set");
    let cert_name = env::var("CERT_NAME").expect("CERT_NAME must be set");
    let cert_path = format!("{}/{}", root_path, cert_name);

    // Read the entire PEM file asynchronously.
    let cert_bytes = fs::read(cert_path)
        .await
        .expect("Failed to read certificate");

    // Wrap the bytes in a Cursor so we can create a standard BufReader.
    let mut reader = BufReader::new(Cursor::new(cert_bytes));

    // Collect the certificates into a Vec.
    let certs = certs(&mut reader).expect("Failed to parse certificate");

    // Convert each certificate to a rustls::Certificate and add to the store.
    for cert in certs {
        root_store
            .add(&Certificate(cert.to_vec()))
            .expect("Failed to add certificate to root store");
    }

    root_store
}

/// Load server certificate and private key to build a ServerConfig.
/// This is used when using Cloudflare Origin CA to issue your origin server's certificate.
/// You must have generated your own private key and then obtained a certificate from Cloudflare.
pub async fn load_certificates() -> std::io::Result<ServerConfig> {
    dotenvy::dotenv().ok();
    info!("Loading Certification for Backend");

    // These env vars should point to your server's certificate and private key.
    let server_cert_path =
        env::var("SERVER_CERT_ROOT_PATH").expect("SERVER_CERT_ROOT_PATH must be set");
    let cert_path = env::var("SERVER_CERT_PATH").expect("SERVER_CERT_PATH must be set");
    let key_path = env::var("SERVER_KEY_PATH").expect("SERVER_KEY_PATH must be set");

    // Load the server certificate chain.
    let cert_bytes = fs::read(format!("{}/{}", server_cert_path, cert_path)).await?;
    let mut cert_reader = BufReader::new(Cursor::new(cert_bytes));
    let certs = certs(&mut cert_reader)?;
    let cert_chain: Vec<Certificate> = certs.into_iter().map(|c| Certificate(c.to_vec())).collect();

    // Load the private key using rustls-pki-types.
    let key_bytes = fs::read(format!("{}/{}", server_cert_path, key_path)).await?;
    let mut key_reader = BufReader::new(Cursor::new(key_bytes));
    // The `private_key` function returns a Vec of keys.
    let mut keys = pkcs8_private_keys(&mut key_reader)?;
    let private_key = if !keys.is_empty() {
        //println!("Private Key : {:#?}", keys);
        PrivateKey(keys.remove(0))
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No private keys found",
        ));
    };

    // Build the server config using the certificate chain and private key.
    let tls_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth() // Use with_client_cert_verifier(...) if you need mTLS.
        .with_single_cert(cert_chain, private_key)
        .expect("Failed to create TLS server configuration");

    info!("Backend Certification Success");
    Ok(tls_config)
}
