use viz::middleware::sessions::*;
use viz::middleware::Cookies;
use viz::prelude::{delete, get, post, put, router, Error, Response, Result, Server, State};
use viz::utils::serde::json::json;

/// `curl 127.0.0.1:3000/ -H 'Cookie: viz.sid=x' -vvv`
async fn index(State(session): State<Session<MemoryStorage>>) -> Result<impl Into<Response>> {
    let data = session.data()?;
    Ok(json!(data))
}

/// `curl -X POST 127.0.0.1:3000/login -vvv`
async fn login(State(session): State<Session<MemoryStorage>>) -> Result<impl Into<Response>> {
    session.set::<String>("id", session.id().unwrap());
    session.save().await?;
    Ok("Session Logined")
}

/// `curl -X PUT 127.0.0.1:3000/renew -H 'Cookie: viz.sid=x' -vvv`
async fn renew(State(mut session): State<Session<MemoryStorage>>) -> Result<impl Into<Response>> {
    session.renew().await?;
    Ok("Session Renewed")
}

/// `curl -X DELETE 127.0.0.1:3000/logout -H 'Cookie: viz.sid=x' -vvv`
async fn logout(State(session): State<Session<MemoryStorage>>) -> Result<impl Into<Response>> {
    session.destroy().await?;
    Ok("Session Logouted")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut app = viz::new();

    app.routes(
        router()
            .with(Cookies::default())
            .with(Sessions::new(Config {
                cookie: CookieOptions::new(),
                storage: MemoryStorage::new(),
                generate: Box::new(|| nano_id::base64::<21>()),
                verify: Box::new(|a| a.len() == 21),
            }))
            .at("/", get(index))
            .at("/login", post(login))
            .at("/logout", delete(logout))
            .at("/renew", put(renew)),
    );

    Server::bind(&"127.0.0.1:3000".parse()?).serve(app.into_service()).await.map_err(Error::new)
}
