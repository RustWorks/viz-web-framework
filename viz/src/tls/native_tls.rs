use std::{fmt, net::SocketAddr};

use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{native_tls::TlsAcceptor as TlsAcceptorWrapper, TlsStream};

use super::Listener;
use crate::{Error, Result};

pub use tokio_native_tls::{native_tls::Identity, TlsAcceptor};

/// [`native-tls`]'s config.
pub struct Config {
    identity: Identity,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeTls Config").finish()
    }
}

impl Config {
    /// Creates a new config with the specified [`Identity`].
    #[must_use]
    pub fn new(identity: Identity) -> Self {
        Self { identity }
    }

    /// Creates a new [`TlsAcceptor`] wrapper with the specified [`Identity`].
    ///
    /// # Errors
    ///
    /// Will return `Err` if wrapping the identity fails.
    pub fn build(self) -> Result<TlsAcceptor> {
        TlsAcceptorWrapper::new(self.identity)
            .map(Into::into)
            .map_err(Error::boxed)
    }
}

impl Listener<TcpListener, TlsAcceptor> {
    /// A [`TlsStream`] and [`SocketAddr`] part for accepting TLS.
    ///
    /// # Errors
    ///
    /// Will return `Err` if accepting the stream fails.
    pub async fn accept(&self) -> Result<(TlsStream<TcpStream>, SocketAddr)> {
        let (stream, addr) = self.inner.accept().await?;
        let tls_stream = self.acceptor.accept(stream).await.map_err(Error::boxed)?;
        Ok((tls_stream, addr))
    }
}
