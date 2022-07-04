use crate::{header, Body, Error, Response, Result, StatusCode};

pub trait IntoResponse: Sized {
    fn into_response(self) -> Response<Body>;

    fn into_error(self) -> Error {
        Error::Responder(self.into_response())
    }
}

impl IntoResponse for Response<Body> {
    fn into_response(self) -> Response<Body> {
        self
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        match self {
            Error::Responder(resp) => resp,
            Error::Report(_, resp) => resp,
        }
    }
}

impl IntoResponse for std::io::Error {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(self.to_string().into())
            .unwrap()
    }
}

impl IntoResponse for std::convert::Infallible {
    fn into_response(self) -> Response<Body> {
        Response::new(Body::empty())
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .header(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
            )
            .body(self.into())
            .unwrap()
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .header(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
            )
            .body(self.into())
            .unwrap()
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .header(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(mime::APPLICATION_OCTET_STREAM.as_ref()),
            )
            .body(self.into())
            .unwrap()
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .header(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(mime::APPLICATION_OCTET_STREAM.as_ref()),
            )
            .body(self.into())
            .unwrap()
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(self)
            .body(Body::empty())
            .unwrap()
    }
}

impl<T> IntoResponse for Option<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response<Body> {
        match self {
            Some(r) => r.into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response<Body> {
        match self {
            Ok(r) => r.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response<Body> {
        Response::new(Body::empty())
    }
}

impl<T> IntoResponse for (StatusCode, T)
where
    T: IntoResponse,
{
    fn into_response(self) -> Response<Body> {
        let mut res = self.1.into_response();
        *res.status_mut() = self.0;
        res
    }
}
