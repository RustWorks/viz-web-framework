use std::{
    io,
    path::{Path, PathBuf},
    pin::Pin,
    task::Poll,
    {fs::Metadata, str::FromStr},
};

use bytes::{BufMut, Bytes, BytesMut};
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt,
    IfModifiedSince, IfRange, IfUnmodifiedSince, LastModified, Range,
};
use tokio::{
    fs::{read_dir, File},
    io::AsyncSeekExt,
};
use tokio_util::io::poll_read_buf;

use viz_utils::{
    futures::{
        future::{self, BoxFuture, Either, FutureExt},
        ready,
        stream::{self, Stream, StreamExt},
    },
    tracing,
};

mod template;

use template::*;

use crate::{http, Context, Middleware, Response, Result};

#[derive(Clone, Debug)]
pub struct Config {
    public: PathBuf,
    unlisted: Option<Vec<&'static str>>,
}

impl Config {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { public: path.into(), unlisted: Some(vec![".DS_Store", ".git"]) }
    }

    pub fn unlisted(&mut self) -> Option<&mut Vec<&'static str>> {
        self.unlisted.as_mut()
    }
}

#[derive(Clone, Debug)]
pub struct Serve {
    config: Config,
}

impl Serve {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    async fn send_file(&self, cx: &Context) -> Result {
        let mut path = self.config.public.clone();
        let mut suffix_path = String::new();

        if let Ok(suffix) = cx.params::<String>() {
            suffix_path += &suffix;
            path.push(suffix);
        }

        let file = File::open(&path).await?;
        let metadata = file.metadata().await?;
        let file_type = metadata.file_type();

        if file_type.is_file() {
            return Ok(respond(
                file,
                metadata,
                cx.headers(),
                path.extension()
                    .and_then(|ext| {
                        ext.to_str().and_then(|ext| {
                            mime_db::lookup(ext).and_then(|ext| mime::Mime::from_str(ext).ok())
                        })
                    })
                    .unwrap_or_else(|| mime::APPLICATION_OCTET_STREAM),
            ));
        }

        let is_dir = file_type.is_dir();

        if is_dir {
            let index_file = path.join("index.html");

            if index_file.exists() {
                let file = File::open(index_file.clone()).await?;
                let metadata = file.metadata().await?;

                if metadata.file_type().is_file() {
                    return Ok(respond(file, metadata, cx.headers(), mime::TEXT_HTML_UTF_8));
                }
            }
        }

        let res = if is_dir {
            let body = render(&self.config, path, Path::new(cx.path()), suffix_path).await?;
            let len = body.len();
            let mut res = Response::html(body);
            res.headers_mut().typed_insert(ContentLength(len as u64));
            res
        } else {
            Response::new().with_status(hyper::StatusCode::NOT_FOUND)
        };

        Ok(res)
    }
}

impl<'m> Middleware<'m, Context> for Serve {
    type Output = Result;

    fn call<'a>(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Result> {
        dbg!(&cx);
        let fut = self.send_file(cx);
        Box::pin(async move { fut.await })
    }
}

pub fn serve(config: Config) -> Serve {
    Serve::new(config)
}

fn respond(file: File, metadata: Metadata, headers: &HeaderMap, mime: mime::Mime) -> Response {
    let last_modified = metadata.modified().ok().map(LastModified::from);
    let content_type = ContentType::from(mime);
    let mut len = metadata.len();

    match check_last_modified(last_modified, headers, content_type.to_owned()) {
        Some(res) => res,
        None => {
            if let Some((start, end)) = check_range(last_modified, headers, len) {
                let sub_len = end - start;
                let body = http::Body::wrap_stream(file_stream(file, 8_192, (start, end)));

                let mut res: Response = body.into();

                res.headers_mut().typed_insert(content_type);

                if sub_len != len {
                    *res.status_mut() = http::StatusCode::PARTIAL_CONTENT;
                    res.headers_mut().typed_insert(
                        ContentRange::bytes(start..end, len).expect("valid ContentRange"),
                    );
                    res.headers_mut().typed_insert(
                        ContentRange::bytes(start..end, len).expect("valid ContentRange"),
                    );

                    len = sub_len;
                }

                res.headers_mut().typed_insert(ContentLength(len));
                res.headers_mut().typed_insert(AcceptRanges::bytes());

                if let Some(time) = last_modified {
                    res.headers_mut().typed_insert(time);
                }

                res
            } else {
                let mut res = Response::new().with_status(http::StatusCode::RANGE_NOT_SATISFIABLE);
                res.headers_mut().typed_insert(ContentRange::unsatisfied_bytes(len));
                res
            }
        }
    }
}

fn file_stream(
    mut file: File,
    buf_size: usize,
    (start, end): (u64, u64),
) -> impl Stream<Item = std::result::Result<Bytes, io::Error>> + Send {
    let seek = async move {
        if start != 0 {
            file.seek(io::SeekFrom::Start(start)).await?;
        }
        Ok(file)
    };

    seek.into_stream()
        .map(move |result| {
            let mut buf = BytesMut::new();
            let mut len = end - start;
            let mut f = match result {
                Ok(f) => f,
                Err(f) => return Either::Left(stream::once(future::err(f))),
            };

            Either::Right(stream::poll_fn(move |cx| {
                if len == 0 {
                    return Poll::Ready(None);
                }
                if buf.remaining_mut() < buf_size {
                    buf.reserve(buf_size);
                }
                let n = match ready!(poll_read_buf(Pin::new(&mut f), cx, &mut buf)) {
                    Ok(n) => n as u64,
                    Err(err) => {
                        tracing::debug!("file read error: {}", err);
                        return Poll::Ready(Some(Err(err)));
                    }
                };

                if n == 0 {
                    tracing::debug!("file read found EOF before expected length");
                    return Poll::Ready(None);
                }

                let mut chunk = buf.split().freeze();
                if n > len {
                    chunk = chunk.split_to(len as usize);
                    len = 0;
                } else {
                    len -= n;
                }

                Poll::Ready(Some(Ok(chunk)))
            }))
        })
        .flatten()
}

fn check_last_modified(
    last_modified: Option<LastModified>,
    headers: &HeaderMap,
    content_type: ContentType,
) -> Option<Response> {
    if let Some(since) = headers.typed_get::<IfUnmodifiedSince>() {
        let precondition =
            last_modified.map(|time| since.precondition_passes(time.into())).unwrap_or(false);

        tracing::trace!(
            "if-unmodified-since? {:?} vs {:?} = {}",
            since,
            last_modified,
            precondition
        );

        if !precondition {
            let mut res = Response::new();
            res.headers_mut().typed_insert(content_type);
            *res.status_mut() = http::StatusCode::PRECONDITION_FAILED;
            return Some(res);
        }
    }

    if let Some(since) = headers.typed_get::<IfModifiedSince>() {
        tracing::trace!("if-modified-since? header = {:?}, file = {:?}", since, last_modified);

        let unmodified = last_modified
            .map(|time| !since.is_modified(time.into()))
            // no last_modified means its always modified
            .unwrap_or(false);

        if unmodified {
            let mut res = Response::new();
            res.headers_mut().typed_insert(content_type);
            *res.status_mut() = http::StatusCode::NOT_MODIFIED;
            return Some(res);
        }
    }

    None
}

fn check_range(
    last_modified: Option<LastModified>,
    headers: &HeaderMap,
    max_len: u64,
) -> Option<(u64, u64)> {
    use std::ops::Bound;

    if let Some(if_range) = headers.typed_get::<IfRange>() {
        tracing::trace!("if-range? {:?} vs {:?}", if_range, last_modified);
        let can_range = !if_range.is_modified(None, last_modified.as_ref());

        if !can_range {
            return Some((0, max_len));
        }
    }

    if let Some(range) = headers.typed_get::<Range>() {
        return range
            .iter()
            .map(|(start, end)| {
                let start = match start {
                    Bound::Unbounded => 0,
                    Bound::Included(s) => s,
                    Bound::Excluded(s) => s + 1,
                };

                let end = match end {
                    Bound::Unbounded => max_len,
                    Bound::Included(s) => s + 1,
                    Bound::Excluded(s) => s,
                };

                if start < end && end <= max_len {
                    Some((start, end))
                } else {
                    tracing::trace!("unsatisfiable byte range: {}-{}/{}", start, end, max_len);
                    None
                }
            })
            .next()
            .unwrap_or(Some((0, max_len)));
    }

    Some((0, max_len))
}

async fn render(
    config: &Config,
    path: PathBuf,
    curr_path: &Path,
    suffix_path: String,
) -> Result<String> {
    let mut entries = read_dir(path.clone()).await?;
    let mut files = Vec::new();
    let unlisted = config.unlisted.as_ref();

    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name().into_string().unwrap();

        if unlisted.map_or_else(|| false, |list| list.contains(&file_name.as_ref())) {
            continue;
        }

        let file_type = if entry.file_type().await?.is_file() { FILE } else { FOLDER };

        let file_path = entry.path();
        let file_path = file_path.strip_prefix(path.clone()).unwrap();
        let file_ext = file_path.extension().unwrap_or_default().to_owned().into_string().unwrap();

        files.push((
            curr_path.join(file_path).into_os_string().into_string().unwrap(),
            file_name.to_owned(),
            file_type.to_owned(),
            file_ext,
            file_name,
        ));
    }

    files.sort_by_key(|f| f.1.to_owned());

    if !suffix_path.is_empty() {
        let parent = curr_path.parent().unwrap();
        files.insert(
            0,
            (
                parent.join("").into_os_string().into_string().unwrap(),
                parent.file_name().unwrap().to_owned().into_string().unwrap(),
                DIRECTORY.to_owned(),
                "".to_owned(),
                "..".to_owned(),
            ),
        );
    }

    let mut breadcrumb: Vec<String> = curr_path
        .ancestors()
        .filter(|a| a.file_name().is_some())
        .map(|a| {
            render_breadcrumb(
                a.join("").to_str().unwrap(),
                a.file_name().unwrap().to_str().unwrap(),
            )
        })
        .collect();

    breadcrumb.reverse();

    let body = render_directory(
        curr_path.to_str().unwrap(),
        &breadcrumb.join(" "),
        &files
            .iter()
            .map(|f| {
                render_file(
                    f.0.to_owned(),
                    f.1.to_owned(),
                    f.2.to_owned(),
                    f.3.to_owned(),
                    f.4.to_owned(),
                )
            })
            .collect::<Vec<String>>()
            .join(""),
    );

    Ok(body)
}
