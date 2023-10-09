use std::{net::SocketAddr, sync::Arc};

use hyper_util::{rt::TokioExecutor, server::conn::auto::Builder};
use tokio::io::{AsyncRead, AsyncWrite};

use viz_core::{Io, Result};
use viz_router::Tree;

use crate::Responder;

/// Serve the connections.
///
/// # Errors
///
/// Will return `Err` if the connection does not be served.
pub async fn serve<I>(stream: I, tree: Arc<Tree>, addr: Option<SocketAddr>) -> Result<()>
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    Builder::new(TokioExecutor::new())
        .serve_connection_with_upgrades(Io::new(stream).into_inner(), Responder::new(tree, addr))
        .await
        .map_err(Into::into)
}
