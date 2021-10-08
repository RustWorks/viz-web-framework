use std::{collections::HashMap, future::Future, pin::Pin};

use viz_core::{
    http::{
        header::{HeaderValue, WWW_AUTHENTICATE},
        headers::{authorization, HeaderMapExt},
        StatusCode,
    },
    Context, Middleware, Response, Result,
};

use viz_utils::tracing;

/// Basic Auth Middleware
#[derive(Debug)]
pub struct Basic {
    users: HashMap<String, String>,
    realm: String,
}

impl Default for Basic {
    fn default() -> Self {
        Self::new()
    }
}

impl Basic {
    const INVALID: &'static str = "invalid authorization header";

    /// Creates new `Basic`
    pub fn new() -> Self {
        Self { users: HashMap::new(), realm: String::from("Restricted") }
    }

    /// Creates new `Basic` with users
    pub fn users(mut self, users: HashMap<String, String>) -> Self {
        self.users = users;
        self
    }

    /// Creates new `Basic` with realm
    pub fn realm(mut self, realm: String) -> Self {
        self.realm = realm;
        self
    }

    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let verified = cx
            .headers()
            .typed_get::<authorization::Authorization<authorization::Basic>>()
            .and_then(|auth| {
                self.users.get(auth.0.username()).filter(|password| *password == auth.0.password())
            })
            .is_some();

        tracing::trace!(" {:>7}", verified);

        if verified {
            return cx.next().await;
        }

        let mut res: Response = StatusCode::UNAUTHORIZED.into();

        if self.realm.len() > 0 {
            let mut value = String::with_capacity(8 + self.realm.len());
            value.push_str("realm=\"");
            value.push_str(&self.realm);
            value.push('"');
            res.headers_mut().insert(WWW_AUTHENTICATE, HeaderValue::from_str(&value)?);
        } else {
            res.headers_mut().insert(WWW_AUTHENTICATE, HeaderValue::from_static(Self::INVALID));
        }

        Ok(res)
    }
}

impl<'a> Middleware<'a, Context> for Basic {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}
