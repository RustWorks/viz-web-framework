//! Server-Sent Events (SSE)
//! Thanks: https://github.com/seanmonstar/warp

use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display, Formatter, Write},
    future::Future,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
    time::Duration,
};

use self::sealed::SseError;
use http::header::{HeaderValue, CACHE_CONTROL, CONTENT_TYPE};
use pin_project::pin_project;
use serde::Serialize;
use tokio::time::{self, Sleep};

use viz_utils::{
    futures::{
        future,
        stream::{Stream, TryStream, TryStreamExt},
    },
    serde::json as serde_json,
    tracing,
};

use crate::Result;

/// Server-sent event data type
#[derive(Debug)]
enum DataType {
    Text(String),
    Json(String),
}

/// Server-sent event
#[derive(Default, Debug)]
pub struct Event {
    name: Option<String>,
    id: Option<String>,
    data: Option<DataType>,
    event: Option<String>,
    comment: Option<String>,
    retry: Option<Duration>,
}

impl Event {
    /// Set Server-sent event data
    /// data field(s) ("data:<content>")
    pub fn data<T: Into<String>>(mut self, data: T) -> Event {
        self.data = Some(DataType::Text(data.into()));
        self
    }

    /// Set Server-sent event data
    /// data field(s) ("data:<content>")
    pub fn json_data<T: Serialize>(mut self, data: T) -> Result<Event> {
        self.data = Some(DataType::Json(serde_json::to_string(&data)?));
        Ok(self)
    }

    /// Set Server-sent event comment
    /// Comment field (":<comment-text>")
    pub fn comment<T: Into<String>>(mut self, comment: T) -> Event {
        self.comment = Some(comment.into());
        self
    }

    /// Set Server-sent event event
    /// Event name field ("event:<event-name>")
    pub fn event<T: Into<String>>(mut self, event: T) -> Event {
        self.event = Some(event.into());
        self
    }

    /// Set Server-sent event retry
    /// Retry timeout field ("retry:<timeout>")
    pub fn retry(mut self, duration: Duration) -> Event {
        self.retry = Some(duration);
        self
    }

    /// Set Server-sent event id
    /// Identifier field ("id:<identifier>")
    pub fn id<T: Into<String>>(mut self, id: T) -> Event {
        self.id = Some(id.into());
        self
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ref comment) = &self.comment {
            ":".fmt(f)?;
            comment.fmt(f)?;
            f.write_char('\n')?;
        }

        if let Some(ref event) = &self.event {
            "event:".fmt(f)?;
            event.fmt(f)?;
            f.write_char('\n')?;
        }

        match self.data {
            Some(DataType::Text(ref data)) => {
                for line in data.split('\n') {
                    "data:".fmt(f)?;
                    line.fmt(f)?;
                    f.write_char('\n')?;
                }
            }
            Some(DataType::Json(ref data)) => {
                "data:".fmt(f)?;
                data.fmt(f)?;
                f.write_char('\n')?;
            }
            None => {}
        }

        if let Some(ref id) = &self.id {
            "id:".fmt(f)?;
            id.fmt(f)?;
            f.write_char('\n')?;
        }

        if let Some(ref duration) = &self.retry {
            "retry:".fmt(f)?;

            let secs = duration.as_secs();
            let millis = duration.subsec_millis();

            if secs > 0 {
                // format seconds
                secs.fmt(f)?;

                // pad milliseconds
                if millis < 10 {
                    f.write_str("00")?;
                } else if millis < 100 {
                    f.write_char('0')?;
                }
            }

            // format milliseconds
            millis.fmt(f)?;

            f.write_char('\n')?;
        }

        f.write_char('\n')?;
        Ok(())
    }
}

/// Gets the optional last event id from request.
/// Typically this identifier represented as number or string.
/// Context Extends
impl crate::Context {
    /// Gets the last event id
    pub fn last_event_id<T>(&self) -> Option<T>
    where
        T: FromStr + Send + Sync + 'static,
    {
        self.header("last-event-id")
    }
}

/// Server-sent events reply
///
/// This function converts stream of server events into a `Reply` with:
///
/// - Status of `200 OK`
/// - Header `content-type: text/event-stream`
/// - Header `cache-control: no-cache`.
pub fn reply<S>(event_stream: S) -> crate::Response
where
    S: TryStream<Ok = Event> + Send + 'static,
    S::Error: StdError + Send + Sync + 'static,
{
    SseResponse { event_stream }.into()
}

#[allow(missing_debug_implementations)]
struct SseResponse<S> {
    event_stream: S,
}

impl<S> From<SseResponse<S>> for crate::Response
where
    S: TryStream<Ok = Event> + Send + 'static,
    S::Error: StdError + Send + Sync + 'static,
{
    fn from(v: SseResponse<S>) -> crate::Response {
        let body_stream = v
            .event_stream
            .map_err(|error| {
                // FIXME: error logging
                tracing::error!("sse stream error: {}", error);
                SseError
            })
            .into_stream()
            .and_then(|event| future::ready(Ok(event.to_string())));

        let mut res = hyper::Response::new(hyper::Body::wrap_stream(body_stream));
        // Set appropriate content type
        res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));
        // Disable response body caching
        res.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        res.into()
    }
}

/// Configure the interval between keep-alive messages, the content
/// of each message, and the associated stream.
#[derive(Debug)]
pub struct KeepAlive {
    comment_text: Cow<'static, str>,
    max_interval: Duration,
}

impl KeepAlive {
    /// Customize the interval between keep-alive messages.
    ///
    /// Default is 15 seconds.
    pub fn interval(mut self, time: Duration) -> Self {
        self.max_interval = time;
        self
    }

    /// Customize the text of the keep-alive message.
    ///
    /// Default is an empty comment.
    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.comment_text = text.into();
        self
    }

    /// Wrap an event stream with keep-alive functionality.
    ///
    /// See [`keep_alive`](keep_alive) for more.
    pub fn stream<S>(
        self,
        event_stream: S,
    ) -> impl TryStream<Ok = Event, Error = impl StdError + Send + Sync + 'static> + Send + 'static
    where
        S: TryStream<Ok = Event> + Send + 'static,
        S::Error: StdError + Send + Sync + 'static,
    {
        let alive_timer = time::sleep(self.max_interval);
        SseKeepAlive {
            event_stream,
            comment_text: self.comment_text,
            max_interval: self.max_interval,
            alive_timer,
        }
    }
}

#[allow(missing_debug_implementations)]
#[pin_project]
struct SseKeepAlive<S> {
    #[pin]
    event_stream: S,
    comment_text: Cow<'static, str>,
    max_interval: Duration,
    #[pin]
    alive_timer: Sleep,
}

/// Keeps event source connection alive when no events sent over a some time.
///
/// Some proxy servers may drop HTTP connection after a some timeout of inactivity.
/// This function helps to prevent such behavior by sending comment events every
/// `keep_interval` of inactivity.
///
/// By default the comment is `:` (an empty comment) and the time interval between
/// events is 15 seconds. Both may be customized using the builder pattern
/// as shown below link.
/// See [notes](https://html.spec.whatwg.org/multipage/server-sent-events.html).
pub fn keep_alive() -> KeepAlive {
    KeepAlive { comment_text: Cow::Borrowed(""), max_interval: Duration::from_secs(15) }
}

impl<S> Stream for SseKeepAlive<S>
where
    S: TryStream<Ok = Event> + Send + 'static,
    S::Error: StdError + Send + Sync + 'static,
{
    type Item = Result<Event, SseError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut pin = self.project();
        match pin.event_stream.try_poll_next(cx) {
            Poll::Pending => match Pin::new(&mut pin.alive_timer).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => {
                    // restart timer
                    pin.alive_timer.reset(tokio::time::Instant::now() + *pin.max_interval);
                    let comment_str = pin.comment_text.clone();
                    let event = Event::default().comment(comment_str);
                    Poll::Ready(Some(Ok(event)))
                }
            },
            Poll::Ready(Some(Ok(event))) => {
                // restart timer
                pin.alive_timer.reset(tokio::time::Instant::now() + *pin.max_interval);
                Poll::Ready(Some(Ok(event)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(error))) => {
                tracing::error!("sse::keep error: {}", error);
                Poll::Ready(Some(Err(SseError)))
            }
        }
    }
}

mod sealed {
    use super::*;

    /// SSE error type
    #[derive(Debug)]
    pub(crate) struct SseError;

    impl Display for SseError {
        fn fmt(&self, f: &mut Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "sse error")
        }
    }

    impl StdError for SseError {}
}
