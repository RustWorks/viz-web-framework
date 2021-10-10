use viz::prelude::{get, router, types::Multipart, Error, Response, Result, Server};
use viz::utils::futures::TryStreamExt;

use std::fs::File;
use tempfile::tempdir;

async fn show_form() -> Response {
    Response::html(
        r#"
        <!doctype html>
        <html>
        <head></head>
        <body>
            <form action="/" method="post" enctype="multipart/form-data">
                <label>Name: <input type="text" name="name"></label>
                <br />
                <label>Upload file: <input type="file" name="file" multiple></label>
                <br />
                <input type="submit" value="Upload files">
            </form>
        </body>
        </html>
        "#,
    )
}

async fn accept_form(mut form: Multipart) -> Result<impl Into<Response>> {
    let dir = tempdir()?;
    let mut txt = String::new();
    txt.push_str(&dir.path().to_string_lossy());
    txt.push_str("\r\n");

    while let Some(mut field) = form.try_next().await? {
        let name = field.name.to_owned();

        if let Some(filename) = &field.filename {
            let filepath = dir.path().join(filename);
            let mut writer = File::create(&filepath)?;
            let bytes = field.copy_to_file(&mut writer).await?;
            txt.push_str(&format!("file {} {}\r\n", name, bytes));
        } else {
            let buffer = field.bytes().await?;
            let bytes = buffer.len() as u64;
            txt.push_str(&format!("text {} {}\r\n", name, bytes));
        }
    }

    Ok(txt)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = viz::new();

    app.routes(router().at("/", get(show_form).post(accept_form)));

    Server::bind(&"127.0.0.1:3000".parse()?).serve(app.into_service()).await.map_err(Error::new)
}
