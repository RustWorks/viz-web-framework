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

    pub async fn hello(State(jinja): State<Environment<'_>>) -> Result {
        let template = jinja.get_template("hello.txt").map_err(|e| anyhow!(e.to_string()))?;
        Ok(template
            .render(&Context { name: "minijinja".into() })
            .map_err(|e| anyhow!(e.to_string()))?
            .into())
    }
}

mod tpl_ramhorns {
    use super::{anyhow, Result};
    use once_cell::sync::Lazy;
    use ramhorns::{Content, Ramhorns};

    #[derive(Content)]
    pub struct Context {
        name: String,
    }

    static RAMHORNS: Lazy<Ramhorns> = Lazy::new(|| {
        Ramhorns::from_folder_with_extension("templates", "txt")
            .map_err(|e| {
                dbg!(&e);
                e
            })
            .unwrap()
    });

    pub async fn hello() -> Result {
        let template = RAMHORNS.get("hello.txt").ok_or(anyhow!("missing template"))?;
        Ok(template.render(&Context { name: "ramhorns".into() }).into())
    }
}

mod tpl_sailfish {
    use super::Result;
    use sailfish::TemplateOnce;

    #[derive(TemplateOnce)]
    #[template(path = "../templates/hello.stpl")]
    struct Hello {
        messages: Vec<String>,
    }

    pub async fn hello() -> Result {
        let ctx = Hello { messages: vec![String::from("Hello"), String::from("sailfish")] };
        Ok(ctx.render_once()?.into())
    }
}
