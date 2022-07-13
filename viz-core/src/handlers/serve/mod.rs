//! Static file serving and directory listing

use std::{
    borrow::Cow,
    collections::Bound,
    io::{Seek, SeekFrom},
    path::{Path, PathBuf},
    str::FromStr,
    time::SystemTime,
};
use tokio::{fs::File, io::AsyncReadExt};
use tokio_util::io::ReaderStream;

use crate::{
    async_trait,
    headers::{
        AcceptRanges, ContentLength, ContentRange, ContentType, ETag, HeaderMap, HeaderMapExt,
        IfMatch, IfModifiedSince, IfNoneMatch, IfUnmodifiedSince, LastModified, Range,
    },
    Body, Handler, IntoResponse, Method, Request, RequestExt, Response, Result, StatusCode,
};

mod error;
mod template;

pub use error::ServeError;
use template::Directory;

#[derive(Clone)]
pub struct ServeFileHandler {
    path: PathBuf,
}

impl ServeFileHandler {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        if !path.exists() {
            panic!("{} not found", path.to_string_lossy());
        }

        Self { path }
    }
}

#[async_trait]
impl Handler<Request<Body>> for ServeFileHandler {
    type Output = Result<Response<Body>>;

    async fn call(&self, req: Request<Body>) -> Self::Output {
        serve(&self.path, req.headers()).await
    }
}

#[derive(Clone)]
pub struct ServeFilesHandler {
    path: PathBuf,
    listing: bool,
    unlisted: Option<Vec<&'static str>>,
}

impl ServeFilesHandler {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        if !path.exists() {
            panic!("{} not found", path.to_string_lossy());
        }

        Self {
            path,
            listing: false,
            unlisted: None,
        }
    }

    pub fn listing(mut self, listing: bool) -> Self {
        self.listing = listing;
        self
    }

    pub fn unlisted(mut self, unlisted: Vec<&'static str>) -> Self {
        self.unlisted.replace(unlisted);
        self
    }
}

#[async_trait]
impl Handler<Request<Body>> for ServeFilesHandler {
    type Output = Result<Response<Body>>;

    async fn call(&self, req: Request<Body>) -> Self::Output {
        if req.method() != Method::GET {
            Err(ServeError::MethodNotAllowed)?;
        }

        let mut prev = false;
        let mut path = self.path.clone();

        if let Some(param) = req.params::<String>().ok() {
            let p = percent_encoding::percent_decode_str(param.as_str())
                .decode_utf8()
                .map_err(|_| ServeError::InvalidPath)?;
            sanitize_path(&mut path, &p)?;
            prev = true;
        }

        if !path.exists() {
            Err(StatusCode::NOT_FOUND.into_error())?;
        }

        if path.is_file() {
            return serve(&path, req.headers()).await;
        }

        let index = path.join("index.html");
        if index.exists() {
            return serve(&index, req.headers()).await;
        }

        if self.listing {
            return Directory::render(req.path(), prev, &path, &self.unlisted)
                .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR.into_error())
                .map(IntoResponse::into_response);
        }

        Ok(StatusCode::NOT_FOUND.into_response())
    }
}

fn sanitize_path<'a>(path: &'a mut PathBuf, p: &Cow<'a, str>) -> Result<()> {
    for seg in p.split('/') {
        if seg.starts_with("..") {
            return Err(StatusCode::NOT_FOUND.into_error());
        } else if seg.contains('\\') {
            return Err(StatusCode::NOT_FOUND.into_error());
        } else {
            path.push(seg);
        }
    }
    Ok(())
}

fn extract_etag(mtime: &SystemTime, size: u64) -> Option<ETag> {
    ETag::from_str(&format!(
        r#""{}-{}""#,
        mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_millis(),
        size
    ))
    .ok()
}

async fn serve(path: &Path, headers: &HeaderMap) -> Result<Response<Body>> {
    let mut file = std::fs::File::open(path).map_err(ServeError::Io)?;
    let metadata = file
        .metadata()
        .map_err(|_| StatusCode::NOT_FOUND.into_error())?;

    let mut etag = None;
    let mut last_modified = None;
    let mut content_range = None;
    let mut max = metadata.len();

    if let Ok(modified) = metadata.modified() {
        etag = extract_etag(&modified, max);

        if matches!((headers.typed_get::<IfMatch>(), &etag), (Some(if_match), Some(etag)) if !if_match.precondition_passes(etag))
            || matches!(headers.typed_get::<IfUnmodifiedSince>(), Some(if_unmodified_since) if !if_unmodified_since.precondition_passes(modified))
        {
            Err(ServeError::PreconditionFailed)?;
        }

        if matches!((headers.typed_get::<IfNoneMatch>(), &etag), (Some(if_no_match), Some(etag)) if !if_no_match.precondition_passes(etag))
            || matches!(headers.typed_get::<IfModifiedSince>(), Some(if_modified_since) if !if_modified_since.is_modified(modified))
        {
            return Ok(StatusCode::NOT_MODIFIED.into_response());
        }

        last_modified.replace(LastModified::from(modified));
    }

    if let Some((start, end)) = headers
        .typed_get::<Range>()
        .and_then(|range| range.iter().next())
    {
        let start = match start {
            Bound::Included(n) => n,
            Bound::Excluded(n) => n + 1,
            Bound::Unbounded => 0,
        };
        let end = match end {
            Bound::Included(n) => n + 1,
            Bound::Excluded(n) => n,
            Bound::Unbounded => max,
        };

        if end < start || end > max {
            Err(ServeError::RangeUnsatisfied(max))?;
        }

        if start != 0 || end != max {
            if let Ok(range) = ContentRange::bytes(start..end, max) {
                max = end - start;
                content_range.replace(range);
                file.seek(SeekFrom::Start(start)).map_err(ServeError::Io)?;
            }
        }
    }

    let body = if content_range.is_some() {
        // max = end - start
        Body::wrap_stream(ReaderStream::new(File::from_std(file).take(max)))
    } else {
        Body::wrap_stream(ReaderStream::new(File::from_std(file)))
    };

    Response::builder()
        .body(body)
        .map(|mut res| {
            let headers = res.headers_mut();

            headers.typed_insert(AcceptRanges::bytes());
            headers.typed_insert(ContentLength(max));
            headers.typed_insert(ContentType::from(
                mime_guess::from_path(path).first_or_octet_stream(),
            ));

            if let Some(etag) = etag {
                headers.typed_insert(etag);
            }

            if let Some(last_modified) = last_modified {
                headers.typed_insert(last_modified);
            }

            if let Some(content_range) = content_range {
                headers.typed_insert(content_range);
                *res.status_mut() = StatusCode::PARTIAL_CONTENT;
            };

            res
        })
        .map_err(Into::into)
}
