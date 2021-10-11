use serde::Deserialize;
use viz::prelude::{get, router, Error, Form, Response, Result, Server};

#[derive(Debug, Deserialize)]
struct Data {
    username: String,
    password: String,
}

impl Into<Response> for Data {
    fn into(self) -> Response {
        Response::text(format!("{} {}", self.username, self.password))
    }
}

async fn show_form() -> Response {
    Response::html(
        r#"
        <!doctype html>
        <html>
        <head></head>
        <body>
            <form action="/" method="post">
                <label>Username: <input type="text" name="username"></label>
                <br />
                <label>Password: <input type="password" name="password"></label>
                <br />
                <input type="submit" value="Save">
            </form>
        </body>
        </html>
        "#,
    )
}

async fn accept_form(Form(data): Form<Data>) -> impl Into<Response> {
    data
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = viz::new();

    app.routes(router().at("/", get(show_form).post(accept_form)));

    Server::bind(&"127.0.0.1:3000".parse()?).serve(app.into_service()).await.map_err(Error::new)
}
