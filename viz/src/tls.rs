//! Tls

use std::{
    future::{self, Future, Ready},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
    {fmt, sync::Arc},
};

use hyper::service::Service;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::{
    rustls::{
        server::{
            AllowAnyAnonymousOrAuthenticatedClient, AllowAnyAuthenticatedClient, NoClientAuth,
        },
        Certificate, PrivateKey, RootCertStore, ServerConfig,
    },
    TlsAcceptor,
};

use viz_utils::{
    anyhow::{anyhow, Result},
    futures::ready,
};

#[cfg(feature = "tcp")]
pub use hyper::server::conn::AddrIncoming;
#[cfg(all(unix, feature = "uds"))]
pub use tokio::net::UnixListener;

use crate::app::{App, AppStream, IntoService};

/// Tls client authentication configuration.
pub(crate) enum ClientAuth {
    /// No client auth.
    Off,
    /// Allow any anonymous or authenticated client.
    Optional(Vec<u8>),
    /// Allow any authenticated client.
    Required(Vec<u8>),
}

/// Tls Config
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
            .map_err(|_| anyhow!("failed to parse tls certificates"))?;

        let keys = {
            let mut pkcs8: Vec<PrivateKey> =
                rustls_pemfile::pkcs8_private_keys(&mut self.key.as_slice())
                    .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
                    .map_err(|_| anyhow!("failed to parse tls private keys"))?;
            if !pkcs8.is_empty() {
                pkcs8.remove(0)
            } else {
                let mut rsa: Vec<PrivateKey> =
                    rustls_pemfile::rsa_private_keys(&mut self.key.as_slice())
                        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
                        .map_err(|_| anyhow!("failed to parse tls private keys"))?;

                if !rsa.is_empty() {
                    rsa.remove(0)
                } else {
                    return Err(anyhow!("failed to parse tls private keys"));
                }
            }
        };

        fn read_trust_anchor(trust_anchor: &Certificate) -> Result<RootCertStore> {
            let mut store = RootCertStore::empty();
            store.add(trust_anchor)?;
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
            .map_err(Into::into)
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config").finish()
    }
}

/// TLS Listener Wrapper
pub struct Listener<T> {
    inner: T,
    acceptor: TlsAcceptor,
}

impl<T> Listener<T> {
    /// Creates new Listener Wrapper
    pub fn new(inner: T, tls_config: ServerConfig) -> Self {
        Self { inner, acceptor: tokio_rustls::TlsAcceptor::from(Arc::new(tls_config)) }
    }
}

impl<T> AsRef<T> for Listener<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> AsMut<T> for Listener<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

/// TLS Stream
pub struct Stream<IO> {
    state: State<IO>,
    remote_addr: Option<SocketAddr>,
}

impl<IO> Stream<IO> {
    fn new(s: tokio_rustls::Accept<IO>, remote_addr: Option<SocketAddr>) -> Self {
        Self { state: State::Handshaking(s), remote_addr }
    }
}

impl<IO> fmt::Debug for Stream<IO> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stream").field("state", &self.state).finish()
    }
}

/// TLS State
enum State<IO> {
    Handshaking(tokio_rustls::Accept<IO>),
    Streaming(tokio_rustls::server::TlsStream<IO>),
}

impl<IO> fmt::Debug for State<IO> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            &(match self {
                Self::Handshaking(_) => "handshaking",
                Self::Streaming(_) => "streaming",
            }),
        )
    }
}

impl<T> fmt::Debug for Listener<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Listener").finish()
    }
}

impl<IO> AsyncRead for Stream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_read(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl<IO> AsyncWrite for Stream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_write(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

#[cfg(feature = "tcp")]
impl Service<&Stream<hyper::server::conn::AddrStream>> for IntoService<App> {
    type Response = AppStream;
    type Error = std::convert::Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, t: &Stream<hyper::server::conn::AddrStream>) -> Self::Future {
        future::ready(Ok(AppStream::new(self.service.clone(), t.remote_addr)))
    }
}

#[cfg(feature = "tcp")]
impl hyper::server::accept::Accept for Listener<hyper::server::conn::AddrIncoming> {
    type Conn = Stream<hyper::server::conn::AddrStream>;
    type Error = std::io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match ready!(Pin::new(&mut self.inner).poll_accept(cx)) {
            Some(Ok(sock)) => Poll::Ready(Some(Ok({
                let remote_addr = sock.remote_addr();
                Stream::new(self.acceptor.accept(sock), Some(remote_addr))
            }))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

#[cfg(all(unix, feature = "uds"))]
impl Service<&Stream<tokio::net::UnixStream>> for IntoService<App> {
    type Response = AppStream;
    type Error = std::convert::Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, t: &Stream<tokio::net::UnixStream>) -> Self::Future {
        future::ready(Ok(AppStream::new(self.service.clone(), t.remote_addr)))
    }
}

#[cfg(all(unix, feature = "uds"))]
impl hyper::server::accept::Accept for Listener<tokio::net::UnixListener> {
    type Conn = Stream<tokio::net::UnixStream>;
    type Error = std::io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match ready!(Pin::new(&mut self.inner).poll_accept(cx)) {
            Ok((sock, _)) => Poll::Ready(Some(Ok(Stream::new(self.acceptor.accept(sock), None)))),
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}
