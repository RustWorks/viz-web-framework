use http::HeaderValue;
use viz_core::{header::FORWARDED, types::RealIp, Request, RequestExt, Result};

#[test]
fn realip() -> Result<()> {
    let mut req = Request::default();
    req.headers_mut()
        .insert(RealIp::X_REAL_IP, HeaderValue::from_static("10.10.10.10"));
    assert_eq!(req.realip(), Some(RealIp("10.10.10.10".parse().unwrap())));

    let mut req = Request::default();
    req.headers_mut().insert(
        RealIp::X_FORWARDED_FOR,
        HeaderValue::from_static("10.10.10.10"),
    );
    assert_eq!(req.realip(), Some(RealIp("10.10.10.10".parse().unwrap())));

    let mut req = Request::default();
    req.headers_mut()
        .insert(FORWARDED, HeaderValue::from_static("for=10.10.10.10"));
    assert_eq!(req.realip(), Some(RealIp("10.10.10.10".parse().unwrap())));

    Ok(())
}
