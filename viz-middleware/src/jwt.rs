//! JSON Web Token Middleware

use std::{fmt::Debug, future::Future, marker::PhantomData, pin::Pin};

use viz_core::{
    http::{
        header::{HeaderValue, WWW_AUTHENTICATE},
        StatusCode,
    },
    Context, Middleware, Response, Result,
};

use viz_utils::tracing;

#[cfg(feature = "jwt-header")]
use viz_core::http::headers::{
    authorization::{Authorization, Bearer},
    HeaderMapExt,
};

#[cfg(all(
    feature = "jwt-query",
    not(all(feature = "jwt-header", feature = "jwt-param", feature = "jwt-cookie"))
))]
use std::collections::HashMap;

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::de::DeserializeOwned;

pub use jsonwebtoken;

/// JWT Middleware
#[derive(Debug)]
pub struct Jwt<T>
where
    T: Debug,
{
    #[cfg(all(
        not(feature = "jwt-header"),
        any(feature = "jwt-query", feature = "jwt-param", feature = "jwt-cookie")
    ))]
    n: String,
    s: String,
    v: Validation,
    t: PhantomData<T>,
}

impl<T> Jwt<T>
where
    T: DeserializeOwned + Sync + Send + 'static + Debug,
{
    /// Creates JWT
    pub fn new() -> Self {
        Self {
            #[cfg(all(
                not(feature = "jwt-header"),
                any(feature = "jwt-query", feature = "jwt-param", feature = "jwt-cookie")
            ))]
            n: "token".to_owned(),
            s: "secret".to_owned(),
            v: Validation::default(),
            t: PhantomData::default(),
        }
    }

    /// Creates JWT Middleware with a secret
    pub fn secret(mut self, secret: &str) -> Self {
        self.s = secret.to_owned();
        self
    }

    /// Creates JWT Middleware with an validation
    pub fn validation(mut self, validation: Validation) -> Self {
        self.v = validation;
        self
    }

    /// Creates JWT Middleware with a name
    #[cfg(all(
        not(feature = "jwt-header"),
        any(feature = "jwt-query", feature = "jwt-param", feature = "jwt-cookie")
    ))]
    pub fn name(mut self, name: &str) -> Self {
        self.n = name.to_owned();
        self
    }

    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let (status, error) = if let Some(val) = self.get(cx) {
            match decode::<T>(&val, &DecodingKey::from_secret(self.s.as_ref()), &self.v) {
                Ok(token) => {
                    cx.extensions_mut().insert(token);
                    return cx.next().await;
                }
                Err(e) => {
                    tracing::error!("JWT error: {}", e);
                    (StatusCode::UNAUTHORIZED, "Invalid or expired JWT")
                }
            }
        } else {
            (StatusCode::BAD_REQUEST, "Missing or malformed JWT")
        };

        let mut res: Response = status.into();
        res.headers_mut().insert(WWW_AUTHENTICATE, HeaderValue::from_str(error)?);

        Ok(res)
    }

    #[allow(unused_variables)]
    /// Gets token via Header|Query|Param|Cookie.
    fn get(&self, cx: &mut Context) -> Option<String> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "jwt-header")] {
                cx.headers()
                    .typed_get::<Authorization<Bearer>>()
                    .map(|auth| auth.0.token().to_owned())
            } else if #[cfg(feature = "jwt-query")] {
                cx.query::<HashMap<String, String>>()
                    .ok()?
                    .get(&self.n)
                    .cloned()
            } else if #[cfg(feature = "jwt-param")] {
                cx.param(&self.n).ok()
            }  else if #[cfg(feature = "jwt-cookie")] {
                cx.cookie(&self.n).map(std::string::ToString::to_string)
            } else {
                None
            }
        }
    }
}

impl<T> Default for Jwt<T>
where
    T: DeserializeOwned + Sync + Send + 'static + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> Middleware<'a, Context> for Jwt<T>
where
    T: DeserializeOwned + Sync + Send + 'static + Debug,
{
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}
