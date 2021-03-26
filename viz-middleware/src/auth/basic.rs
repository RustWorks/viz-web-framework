use std::{collections::HashMap, future::Future, pin::Pin};

use viz_core::{
    http::{
        header::{HeaderValue, WWW_AUTHENTICATE},
        headers::{
            authorization::{Authorization, Basic},
            HeaderMapExt,
        },
        StatusCode,
    },
    Context, Middleware, Response, Result,
};

use viz_utils::log;

/// Basic Auth Middleware
#[derive(Debug)]
pub struct BasicMiddleware {
    users: HashMap<String, String>,
    realm: String,
}

impl BasicMiddleware {
    /// Creates new `BasicMiddleware`
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            realm: String::from("Restricted"),
        }
    }

    /// Creates new `BasicMiddleware` with users
    pub fn users(mut self, users: HashMap<String, String>) -> Self {
        self.users = users;
        self
    }

    /// Creates new `BasicMiddleware` with realm
    pub fn realm(mut self, realm: String) -> Self {
        self.realm = realm;
        self
    }
}

impl Default for BasicMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicMiddleware {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        log::trace!("Basic Auth Middleware");

        if cx
            .headers()
            .typed_get::<Authorization<Basic>>()
            .and_then(|auth| {
                let user = auth.0.username();
                let pswd = auth.0.password();
                self.users.get(user).filter(|password| *password == pswd)
            })
            .is_some()
        {
            return cx.next().await;
        }

        let mut res: Response = StatusCode::UNAUTHORIZED.into();
        res.headers_mut().insert(
            WWW_AUTHENTICATE,
            HeaderValue::from_str("invalid authorization header")?,
        );
        Ok(res)
    }
}

impl<'a> Middleware<'a, Context> for BasicMiddleware {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}
