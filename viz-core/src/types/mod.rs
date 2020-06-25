mod form;
mod json;
mod multipart;
mod params;
mod payload;

pub use form::{form, Form};
pub use json::{json, Json};
pub use multipart::{multipart, FormData};
pub use params::{Params, ParamsDeserializer};
pub use payload::{get_length, get_mime, Payload, PayloadCheck, PayloadError, PAYLOAD_LIMIT};

#[cfg(test)]
mod tests {
    use viz_utils::futures::stream::{self, TryStreamExt};
    use viz_utils::serde::urlencoded;
    use viz_utils::smol::block_on;

    use bytes::buf::BufExt;
    use serde::Deserialize;

    use crate::*;

    #[derive(Debug, PartialEq, Deserialize)]
    struct Lang {
        name: String,
    }

    #[test]
    fn test_payload_error_into_response() {
        block_on(async move {
            let e = PayloadError::TooLarge;
            let r: Response = e.into();
            assert_eq!(r.raw.status(), 413);

            let (_, body) = r.raw.into_parts();
            assert_eq!(
                hyper::body::to_bytes(body).await.unwrap(),
                "payload is too large"
            );
        });

        block_on(async move {
            let e = PayloadError::Parse;
            let r: Response = e.into();
            assert_eq!(r.raw.status(), 400);

            let (_, body) = r.raw.into_parts();
            assert_eq!(
                hyper::body::to_bytes(body).await.unwrap(),
                "failed to parse payload"
            );
        });
    }

    #[test]
    fn test_payload_parse_json() {
        block_on(async move {
            let chunks: Vec<Result<_, std::io::Error>> =
                vec![Ok(r#"{"name""#), Ok(": "), Ok(r#""rustlang"}"#)];

            let stream = stream::iter(chunks);

            let body = http::Body::wrap_stream(stream);

            let mut req = http::Request::new(body);

            req.headers_mut().insert(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_JSON.to_string().parse().unwrap(),
            );
            req.headers_mut()
                .insert(http::header::CONTENT_LENGTH, "20".parse().unwrap());

            let mut cx: Context = req.into();

            let data = cx.extract::<Json<Lang>>().await.unwrap();

            assert_eq!(
                *data,
                Lang {
                    name: "rustlang".to_owned()
                }
            );
        });

        block_on(async move {
            let chunks: Vec<Result<_, std::io::Error>> =
                vec![Ok(r#"{"name""#), Ok(": "), Ok(r#""rustlang""#)];

            let stream = stream::iter(chunks);

            let body = http::Body::wrap_stream(stream);

            let mut req = http::Request::new(body);

            req.headers_mut().insert(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_JSON.to_string().parse().unwrap(),
            );
            req.headers_mut()
                .insert(http::header::CONTENT_LENGTH, "20".parse().unwrap());

            let cx: Context = req.into();

            let mut payload = json::<Lang>();

            payload.set_limit(19);

            let m = get_mime(&cx);
            let l = get_length(&cx);

            let err = payload.check_header(m, l).err().unwrap();

            let res = Into::<Response>::into(err).raw;

            assert_eq!(res.status(), http::StatusCode::PAYLOAD_TOO_LARGE);
            assert_eq!(
                hyper::body::to_bytes(res.into_parts().1).await.unwrap(),
                "payload is too large"
            );
        });
    }

    #[test]
    fn test_payload_parse_form() {
        block_on(async move {
            let chunks: Vec<Result<_, std::io::Error>> = vec![
                Ok("name"),
                Ok("="),
                Ok("%E4%BD%A0%E5%A5%BD%EF%BC%8C%E4%B8%96%E7%95%8C"),
            ];

            let stream = stream::iter(chunks);

            let body = http::Body::wrap_stream(stream);

            let mut req = http::Request::new(body);

            req.headers_mut().insert(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED
                    .to_string()
                    .parse()
                    .unwrap(),
            );
            req.headers_mut()
                .insert(http::header::CONTENT_LENGTH, "13".parse().unwrap());

            let mut cx: Context = req.into();

            let mut payload = form::<Lang>();

            let m = get_mime(&cx);
            let l = get_length(&cx);

            assert!(payload.check_header(m, l).is_ok());

            payload.replace(
                urlencoded::from_reader(
                    payload
                        .check_real_length(cx.take_body().unwrap())
                        .await
                        .unwrap()
                        .reader(),
                )
                .map(|o| Form(o))
                .unwrap(),
            );

            assert_eq!(
                *payload.take(),
                Lang {
                    name: "你好，世界".to_owned()
                }
            );
        });
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
            req.headers_mut()
                .insert(http::header::CONTENT_LENGTH, "13".parse().unwrap());

            let mut cx: Context = req.into();

            let payload = multipart();

            let m = get_mime(&cx);

            let l = get_length(&cx);

            let m = payload.check_header(m, l).unwrap();

            let charset = m.get_param(mime::CHARSET);
            let boundary = m.get_param(mime::BOUNDARY);

            assert_eq!(charset.unwrap(), "utf-8");
            assert_eq!(boundary.unwrap(), "b78128d03bdc557f");

            let mut form = cx.extract::<FormData<http::Body>>().await.unwrap();

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

            req.extensions_mut()
                .insert::<Params>(vec![("repo", "viz"), ("id", "233")].into());

            let mut cx: Context = req.into();

            let info = cx.extract::<Params<Info>>().await.unwrap();

            assert_eq!(info.repo, "viz");
            assert_eq!(info.id, 233);

            Ok::<_, Error>(())
        })
        .is_ok());
    }
}
