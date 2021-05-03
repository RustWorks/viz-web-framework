//! Web Types

mod cookies;
mod form;
mod json;
mod multipart;
mod params;
mod payload;
mod query;
mod state;

#[cfg(feature = "private-cookies")]
pub use cookie::PrivateJar;

#[cfg(feature = "signed-cookies")]
pub use cookie::SignedJar;

#[cfg(any(feature = "signed-cookies", feature = "private-cookies"))]
pub use cookie::Key;

pub use cookie::{Cookie, CookieJar};
pub use cookies::{Cookies, CookiesError};
pub use form::Form;
pub use json::Json;
pub use multipart::Multipart;
pub use params::{Params, ParamsError};
pub use payload::{Payload, PayloadDetect, PayloadError};
pub use query::Query;
pub use state::{State, StateFactory};

#[cfg(test)]
mod tests {
    use viz_utils::futures::stream::{self, TryStreamExt};
    use viz_utils::serde::urlencoded;

    use bytes::buf::Buf;
    use futures_executor::block_on;
    use serde::Deserialize;

    use crate::*;
    use types::*;

    #[derive(Debug, PartialEq, Deserialize)]
    struct Lang {
        name: String,
    }

    #[test]
    fn test_payload_error_into_response() {
        assert!(block_on(async move {
            let e = PayloadError::TooLarge;
            let r: Response = e.into();
            assert_eq!(r.raw.status(), 413);

            let (_, body) = r.raw.into_parts();
            assert_eq!(hyper::body::to_bytes(body).await?, "payload is too large");

            Ok::<_, Error>(())
        })
        .is_ok());

        assert!(block_on(async move {
            let e = PayloadError::Parse;
            let r: Response = e.into();
            assert_eq!(r.raw.status(), 400);

            let (_, body) = r.raw.into_parts();
            assert_eq!(hyper::body::to_bytes(body).await?, "failed to parse payload");

            Ok::<_, Error>(())
        })
        .is_ok());
    }

    #[test]
    fn test_payload_parse_json() {
        assert!(block_on(async move {
            let chunks: Vec<Result<_, std::io::Error>> =
                vec![Ok(r#"{"name""#), Ok(": "), Ok(r#""rustlang"}"#)];

            let stream = stream::iter(chunks);

            let body = http::Body::wrap_stream(stream);

            let mut req = http::Request::new(body);

            req.headers_mut()
                .insert(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string().parse()?);
            req.headers_mut().insert(http::header::CONTENT_LENGTH, "20".parse()?);

            let mut cx = Context::from(req);

            let data = cx.extract::<Json<Lang>>().await?;

            assert_eq!(*data, Lang { name: "rustlang".to_owned() });

            Ok::<_, Error>(())
        })
        .is_ok());

        assert!(block_on(async move {
            let chunks: Vec<Result<_, std::io::Error>> =
                vec![Ok(r#"{"name""#), Ok(": "), Ok(r#""rustlang""#)];

            let stream = stream::iter(chunks);

            let body = http::Body::wrap_stream(stream);

            let mut req = http::Request::new(body);

            req.headers_mut()
                .insert(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string().parse()?);
            req.headers_mut().insert(http::header::CONTENT_LENGTH, "20".parse()?);

            let cx = Context::from(req);

            let mut payload = Payload::<Json>::new();

            payload.set_limit(19);

            let err = payload.check_header(cx.mime(), cx.len()).err().unwrap();

            assert_eq!(err, PayloadError::TooLarge);

            let res = Into::<Response>::into(err).raw;

            assert_eq!(res.status(), http::StatusCode::PAYLOAD_TOO_LARGE);
            assert_eq!(hyper::body::to_bytes(res.into_parts().1).await?, "payload is too large");

            Ok::<_, Error>(())
        })
        .is_ok());
    }

    #[test]
    fn test_payload_parse_form() {
        assert!(block_on(async move {
            let chunks: Vec<Result<_, std::io::Error>> =
                vec![Ok("name"), Ok("="), Ok("%E4%BD%A0%E5%A5%BD%EF%BC%8C%E4%B8%96%E7%95%8C")];

            let stream = stream::iter(chunks);

            let body = http::Body::wrap_stream(stream);

            let mut req = http::Request::new(body);

            req.headers_mut().insert(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.to_string().parse()?,
            );
            req.headers_mut().insert(http::header::CONTENT_LENGTH, "13".parse()?);

            let mut cx = Context::from(req);

            let payload = Payload::<Form>::new();

            assert!(payload.check_header(cx.mime(), cx.len()).is_ok());

            let data: Form<Lang> = urlencoded::from_reader(
                payload.check_real_length(cx.take_body().unwrap()).await?.reader(),
            )
            .map(Form)
            .unwrap();

            assert_eq!(*data, Lang { name: "你好，世界".to_owned() });

            Ok::<_, Error>(())
        })
        .is_ok());

        assert!(block_on(async move {
            let chunks: Vec<Result<_, std::io::Error>> =
                vec![Ok("name"), Ok("="), Ok("%E4%BD%A0%E5%A5%BD%EF%BC%8C%E4%B8%96%E7%95%8C")];

            let stream = stream::iter(chunks);

            let body = http::Body::wrap_stream(stream);

            let mut req = http::Request::new(body);

            req.headers_mut().insert(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.to_string().parse()?,
            );
            req.headers_mut().insert(http::header::CONTENT_LENGTH, "13".parse()?);

            let mut cx = Context::from(req);

            let lang: Lang = cx.form().await?;

            assert_eq!(lang, Lang { name: "你好，世界".to_owned() });

            Ok::<_, Error>(())
        })
        .is_ok());
    }

    #[test]
    fn test_payload_parse_multilpart() {
        assert!(block_on(async move {
            let chunks: Vec<Result<_, std::io::Error>> = vec![
                Ok("--b78128d03bdc557f\r\n"),
                Ok("Content-Disposition: form-data; name=\"crate\"\r\n"),
                Ok("\r\n"),
                Ok("form-data\r\n"),
                Ok("--b78128d03bdc557f--\r\n"),
            ];

            let stream = stream::iter(chunks);

            let body = http::Body::wrap_stream(stream);

            let mut req = http::Request::new(body);

            req.headers_mut().insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static(
                    r#"multipart/form-data; charset=utf-8; boundary="b78128d03bdc557f""#,
                ),
            );
            req.headers_mut().insert(http::header::CONTENT_LENGTH, "13".parse()?);

            let mut cx = Context::from(req);

            let payload = Payload::<Multipart>::new();

            let m = payload.check_header(cx.mime(), cx.len())?;

            let charset = m.get_param(mime::CHARSET);
            let boundary = m.get_param(mime::BOUNDARY);

            assert_eq!(charset.unwrap(), "utf-8");
            assert_eq!(boundary.unwrap(), "b78128d03bdc557f");

            let mut form = cx.extract::<Multipart>().await?;

            while let Some(mut field) = form.try_next().await? {
                let buffer = field.bytes().await?;
                assert_eq!(buffer.len(), 9);
                assert_eq!(buffer, b"form-data".to_vec());
            }

            Ok::<_, Error>(())
        })
        .is_ok());

        assert!(block_on(async move {
            let chunks: Vec<Result<_, std::io::Error>> = vec![
                Ok("--b78128d03bdc557f\r\n"),
                Ok("Content-Disposition: form-data; name=\"crate\"\r\n"),
                Ok("\r\n"),
                Ok("form-data\r\n"),
                Ok("--b78128d03bdc557f--\r\n"),
            ];

            let stream = stream::iter(chunks);

            let body = http::Body::wrap_stream(stream);

            let mut req = http::Request::new(body);

            req.headers_mut().insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static(
                    r#"multipart/form-data; charset=utf-8; boundary="b78128d03bdc557f""#,
                ),
            );
            req.headers_mut().insert(http::header::CONTENT_LENGTH, "13".parse()?);

            let mut cx = Context::from(req);

            let mut form = cx.multipart()?;

            while let Some(mut field) = form.try_next().await? {
                let buffer = field.bytes().await?;
                assert_eq!(buffer.len(), 9);
                assert_eq!(buffer, b"form-data".to_vec());
            }

            Ok::<_, Error>(())
        })
        .is_ok());
    }

    #[test]
    fn test_payload_parse_params() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Info {
            repo: String,
            id: u32,
        }

        assert!(block_on(async move {
            let mut req = http::Request::new(http::Body::empty());

            req.extensions_mut().insert::<Params>(vec![("repo", "viz"), ("id", "233")].into());

            let mut cx = Context::from(req);

            let info: Info = cx.params::<Info>()?;
            assert_eq!(info.repo, "viz");
            assert_eq!(info.id, 233);

            let repo: String = cx.param("repo")?;
            assert_eq!(repo, "viz");

            let id: usize = cx.param("id")?;
            assert_eq!(id, 233);

            let info: Params<Info> = cx.extract::<Params<Info>>().await?;
            assert_eq!(info.repo, "viz");
            assert_eq!(info.id, 233);

            Ok::<_, Error>(())
        })
        .is_ok());
    }

    #[test]
    fn test_state() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        assert!(block_on(async move {
            let mut req = http::Request::new(http::Body::empty());

            req.extensions_mut().insert::<State<String>>(State::new("Hey Viz".to_string()));

            let mut cx = Context::from(req);

            let text: String = cx.state()?;
            assert_eq!(text.as_str(), "Hey Viz");

            let text = cx.extract::<State<String>>().await?;
            assert_eq!(text.as_ref(), "Hey Viz");

            Ok::<_, Error>(())
        })
        .is_ok());

        assert!(block_on(async move {
            let mut req = http::Request::new(http::Body::empty());

            let num = Arc::new(AtomicUsize::new(0));

            req.extensions_mut().insert::<State<Arc<AtomicUsize>>>(State::new(num.clone()));

            num.fetch_add(1, Ordering::SeqCst);

            let mut cx = Context::from(req);

            let num_cloned = cx.extract::<State<Arc<AtomicUsize>>>().await?;

            assert_eq!(num_cloned.as_ref().load(Ordering::SeqCst), 1);

            num.fetch_sub(1, Ordering::SeqCst);

            assert_eq!(num.load(Ordering::SeqCst), 0);

            Ok::<_, Error>(())
        })
        .is_ok());
    }

    #[test]
    fn test_cookies() {
        assert!(block_on(async move {
            let mut req = http::Request::new(http::Body::empty());

            req.headers_mut().insert(
                http::header::COOKIE,
                http::HeaderValue::from_static("foo=bar; logged_in=true"),
            );

            let mut cx = Context::from(req);

            let cookies = cx.extract::<Cookies>().await?;

            let cookie = cookies.get("foo").unwrap();
            assert_eq!(cookie.value(), "bar");

            let cookie = cookies.get("logged_in").unwrap();
            assert_eq!(cookie.value(), "true");

            Ok::<_, Error>(())
        })
        .is_ok());

        assert!(block_on(async move {
            let mut req = http::Request::new(http::Body::empty());

            req.headers_mut().insert(
                http::header::COOKIE,
                http::HeaderValue::from_static("foo=bar; logged_in=true"),
            );

            let mut cx = Context::from(req);

            let cookies = cx.cookies()?;

            let cookie = cookies.get("foo").unwrap();
            assert_eq!(cookie.value(), "bar");

            let cookie = cookies.get("logged_in").unwrap();
            assert_eq!(cookie.value(), "true");

            let cookie = cx.cookie("foo").unwrap();
            assert_eq!(cookie.value(), "bar");

            let cookie = cx.cookie("logged_in").unwrap();
            assert_eq!(cookie.value(), "true");

            Ok::<_, Error>(())
        })
        .is_ok());
    }

    #[test]
    fn test_query() {
        assert!(block_on(async move {
            #[derive(Debug, Deserialize, PartialEq)]
            struct Args {
                foo: String,
                crab: usize,
                logged_in: bool,
            }

            let mut req = http::Request::new(http::Body::empty());

            *req.uri_mut() = "/?foo=bar&crab=1&logged_in=true".parse().unwrap();

            let mut cx = Context::from(req);

            assert_eq!(cx.query_str(), "foo=bar&crab=1&logged_in=true");

            let args = cx.query::<Args>()?;
            assert_eq!(args, Args { foo: "bar".to_string(), crab: 1, logged_in: true });

            let args = cx.extract::<Query<Args>>().await?;
            assert_eq!(*args, Args { foo: "bar".to_string(), crab: 1, logged_in: true });

            Ok::<_, Error>(())
        })
        .is_ok());
    }
}
