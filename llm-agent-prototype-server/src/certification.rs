use anyhow::Result;
use std::env;
use std::fs;
use tonic::transport::Identity;
use tonic::transport::ServerTlsConfig;

#[tracing::instrument(name = "tls_config", err)]
pub(crate) fn build_tls_config() -> Result<ServerTlsConfig> {
    // Get the path to the certificate and private key
    let cert_path = env::var("SERVER_CERT_PATH").map_err(|error| {
        tracing::error!("Failed to get SERVER_CERT_PATH: {:?}", error);
        error
    })?;
    let key_path = env::var("SERVER_KEY_PATH").map_err(|error| {
        tracing::error!("Failed to get SERVER_KEY_PATH: {:?}", error);
        error
    })?;

    // Read the certificate and private key
    let cert = fs::read_to_string(cert_path).map_err(|error| {
        tracing::error!("Failed to read certificate: {:?}", error);
        error
    })?;
    let key = fs::read_to_string(key_path).map_err(|error| {
        tracing::error!("Failed to read private key: {:?}", error);
        error
    })?;

    // Create a tonic::Identity from the certificate and private key
    let identity = Identity::from_pem(cert, key);

    // Create a ServerTlsConfig from the Identity
    let tls_config = ServerTlsConfig::new().identity(identity);

    Ok(tls_config)
}
