use std::error::Error as StdError;
use viz_core::{Error, OutgoingBody, Response, StatusCode};

#[test]
fn error() {
    let e: Error = std::io::Error::last_os_error().into();
    assert_eq!("Undefined error: 0 (os error 0)", e.to_string());
    assert!(e.is::<std::io::Error>());
    assert!(e.downcast::<std::io::Error>().is_ok());
    let e: Error = Error::normal(std::io::Error::last_os_error());
    assert!(e.downcast_ref::<std::io::Error>().is_some());
    let boxed: Box<dyn StdError + Send + Sync> = Box::new(std::io::Error::last_os_error());
    let mut e: Error = boxed.into();
    assert!(e.downcast_mut::<std::io::Error>().is_some());

    let e: Error = (
        std::io::Error::new(std::io::ErrorKind::Other, "error"),
        StatusCode::OK,
    )
        .into();
    assert!(e.is::<std::io::Error>());
    assert!(e.downcast::<std::io::Error>().is_ok());
    let e: Error = (
        std::io::Error::new(std::io::ErrorKind::Other, "error"),
        StatusCode::OK,
    )
        .into();
    assert!(e.downcast_ref::<std::io::Error>().is_some());
    let mut e: Error = (
        std::io::Error::new(std::io::ErrorKind::Other, "error"),
        StatusCode::OK,
    )
        .into();
    assert!(e.downcast_mut::<std::io::Error>().is_some());

    let e = Error::Responder(Response::new(OutgoingBody::Empty));
    assert!(!e.is::<std::io::Error>());
    let e = Error::Responder(Response::new(OutgoingBody::Empty));
    assert!(e.downcast::<std::io::Error>().is_err());
    let e = Error::Responder(Response::new(OutgoingBody::Empty));
    assert!(e.downcast_ref::<std::io::Error>().is_none());
    let mut e = Error::Responder(Response::new(OutgoingBody::Empty));
    assert!(e.downcast_mut::<std::io::Error>().is_none());

    let _: Error = http::Error::from(StatusCode::from_u16(1000).unwrap_err()).into();
}