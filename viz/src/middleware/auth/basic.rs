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
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            realm: String::from("Restricted"),
        }
    }

    pub fn users(mut self, users: HashMap<String, String>) -> Self {
        self.users = users;
        self
    }

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
                self.users
                    .get(auth.0.username())
                    .filter(|password| *password == auth.0.password())
            })
            .is_some()
        {
            return cx.next().await;
        }

        let mut res: Response = StatusCode::UNAUTHORIZED.into();
        res.headers_mut().insert(
            WWW_AUTHENTICATE,
            HeaderValue::from_str(&format!("basic realm={}", self.realm))?,
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
