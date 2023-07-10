use std::{net::IpAddr, str};
use viz_core::{header::FORWARDED, types::RealIP, Request, RequestExt, Result};

#[test]
fn realip() -> Result {
    let mut req = Request::default();
    req.headers_mut().insert(FORWARDED, "10.10.10.10");
    assert_eq!(req.realip(), Realip("10.10.10.10".parse().ok()));

    let mut req = Request::default();
    req.headers_mut().insert(Realip::X_REAL_IP, "10.10.10.10");
    assert_eq!(req.realip(), Realip("10.10.10.10".parse().ok()));

    let mut req = Request::default();
    req.headers_mut()
        .insert(Realip::X_FORWARDED_FOR, "10.10.10.10");
    assert_eq!(req.realip(), Realip("10.10.10.10".parse().ok()));

    Ok(())
}
