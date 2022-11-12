use std::{
    io::{Error as IoError, ErrorKind},
    net::SocketAddr,
};

use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::{
    server::{AllowAnyAnonymousOrAuthenticatedClient, AllowAnyAuthenticatedClient, NoClientAuth},
    Certificate, PrivateKey, RootCertStore, ServerConfig,
};

use super::Listener;
use crate::{Error, Result};

pub use tokio_rustls::{server::TlsStream, TlsAcceptor};

/// Tls client authentication configuration.
#[derive(Debug)]
pub(crate) enum ClientAuth {
    /// No client auth.
    Off,
    /// Allow any anonymous or authenticated client.
    Optional(Vec<u8>),
    /// Allow any authenticated client.
    Required(Vec<u8>),
}

/// `rustls`'s config.
#[derive(Debug)]
pub struct Config {
    cert: Vec<u8>,
    key: Vec<u8>,
    ocsp_resp: Vec<u8>,
    client_auth: ClientAuth,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Create a new Tls config
    pub fn new() -> Self {
        Self {
            cert: Vec::new(),
            key: Vec::new(),
            client_auth: ClientAuth::Off,
            ocsp_resp: Vec::new(),
        }
    }

    /// sets the Tls certificate
    pub fn cert(mut self, cert: impl Into<Vec<u8>>) -> Self {
        self.cert = cert.into();
        self
    }

    /// sets the Tls key
    pub fn key(mut self, key: impl Into<Vec<u8>>) -> Self {
        self.key = key.into();
        self
    }

    /// Sets the trust anchor for optional Tls client authentication
    pub fn client_auth_optional(mut self, trust_anchor: impl Into<Vec<u8>>) -> Self {
        self.client_auth = ClientAuth::Optional(trust_anchor.into());
        self
    }

    /// Sets the trust anchor for required Tls client authentication
    pub fn client_auth_required(mut self, trust_anchor: impl Into<Vec<u8>>) -> Self {
        self.client_auth = ClientAuth::Required(trust_anchor.into());
        self
    }

    /// sets the DER-encoded OCSP response
    pub fn ocsp_resp(mut self, ocsp_resp: impl Into<Vec<u8>>) -> Self {
        self.ocsp_resp = ocsp_resp.into();
        self
    }

    /// builds the Tls ServerConfig
    pub fn build(self) -> Result<ServerConfig> {
        let certs = rustls_pemfile::certs(&mut self.cert.as_slice())
            .map(|mut certs| certs.drain(..).map(Certificate).collect())
            .map_err(Error::normal)?;

        let keys = {
            let mut pkcs8: Vec<PrivateKey> =
                rustls_pemfile::pkcs8_private_keys(&mut self.key.as_slice())
                    .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
                    .map_err(Error::normal)?;
            if !pkcs8.is_empty() {
                pkcs8.remove(0)
            } else {
                let mut rsa: Vec<PrivateKey> =
                    rustls_pemfile::rsa_private_keys(&mut self.key.as_slice())
                        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
                        .map_err(Error::normal)?;

                if !rsa.is_empty() {
                    rsa.remove(0)
                } else {
                    return Err(Error::normal(IoError::new(
                        ErrorKind::InvalidData,
                        "failed to parse tls private keys",
                    )));
                }
            }
        };

        fn read_trust_anchor(trust_anchor: &Certificate) -> Result<RootCertStore> {
            let mut store = RootCertStore::empty();
            store.add(trust_anchor).map_err(Error::normal)?;
            Ok(store)
        }

        let client_auth = match self.client_auth {
            ClientAuth::Off => NoClientAuth::new(),
            ClientAuth::Optional(trust_anchor) => AllowAnyAnonymousOrAuthenticatedClient::new(
                read_trust_anchor(&Certificate(trust_anchor))?,
            ),
            ClientAuth::Required(trust_anchor) => {
                AllowAnyAuthenticatedClient::new(read_trust_anchor(&Certificate(trust_anchor))?)
            }
        };

        ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(client_auth)
            .with_single_cert_with_ocsp_and_sct(certs, keys, self.ocsp_resp, Vec::new())
            .map_err(Error::normal)
    }
}

impl Listener<TcpListener, TlsAcceptor> {
    /// A [`TlsStream`] and [`SocketAddr] part for accepting TLS.
    pub async fn accept(&self) -> Result<(TlsStream<TcpStream>, SocketAddr)> {
        let (stream, addr) = self.inner.accept().await?;
        let tls_stream = self.acceptor.accept(stream).await?;
        Ok((tls_stream, addr))
    }
}
