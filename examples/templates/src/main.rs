use viz::prelude::{anyhow, get, router, Error, Result, Server, State};

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = viz::new();

    app.state(tpl_minijinja::init()?).routes(
        router()
            .at("/minijinja", get(tpl_minijinja::hello))
            .at("/ramhorns", get(tpl_ramhorns::hello))
            .at("/sailfish", get(tpl_sailfish::hello)),
    );

    Server::bind(&"127.0.0.1:3000".parse()?).serve(app.into_service()).await.map_err(Error::new)
}

mod tpl_minijinja {
    use super::{anyhow, Result, State};
    use minijinja::Environment;
    use serde::Serialize;
    use viz::prelude::Response;

    #[derive(Serialize)]
    pub struct Context {
        name: String,
    }

    pub fn init() -> Result<Environment<'static>> {
        let mut env = Environment::new();
        env.add_template("hello.txt", include_str!("../templates/hello.txt"))
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(env)
    }

    pub async fn hello(State(jinja): State<Environment<'_>>) -> Result<impl Into<Response>> {
        let tpl = jinja.get_template("hello.txt").map_err(|e| anyhow!(e.to_string()))?;
        tpl.render(&Context { name: "minijinja".into() })
            .map(Response::text)
            .map_err(|e| anyhow!(e.to_string()))
    }
}

mod tpl_ramhorns {
    use super::{anyhow, Result};
    use once_cell::sync::Lazy;
    use ramhorns::{Content, Ramhorns};
    use viz::prelude::Response;

    #[derive(Content)]
    pub struct Context {
        name: String,
    }

    static RAMHORNS: Lazy<Ramhorns> =
        Lazy::new(|| Ramhorns::from_folder_with_extension("templates", "txt").unwrap());

    pub async fn hello() -> Result<impl Into<Response>> {
        let tpl = RAMHORNS.get("hello.txt").ok_or_else(|| anyhow!("missing template"))?;
        Ok(Response::text(tpl.render(&Context { name: "ramhorns".into() })))
    }
}

mod tpl_sailfish {
    use super::{Error, Result};
    use sailfish::TemplateOnce;
    use viz::prelude::Response;

    #[derive(TemplateOnce)]
    #[template(path = "../templates/hello.stpl")]
    struct Hello {
        messages: Vec<String>,
    }

    pub async fn hello() -> Result<impl Into<Response>> {
        let tpl = Hello { messages: vec![String::from("Hello"), String::from("sailfish")] };
        tpl.render_once().map(Response::html).map_err(Error::new)
    }
}
