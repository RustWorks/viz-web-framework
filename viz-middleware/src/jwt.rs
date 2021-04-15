//! JSON Web Token Middleware

use std::{future::Future, marker::PhantomData, pin::Pin, fmt::Debug};

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

#[cfg(feature = "jwt-query")]
#[cfg(not(all(feature = "jwt-header", feature = "jwt-param", feature = "jwt-cookie")))]
use std::collections::HashMap;
#[cfg(feature = "jwt-query")]
#[cfg(not(all(feature = "jwt-header", feature = "jwt-param", feature = "jwt-cookie")))]
use viz_core::types::QueryContextExt;

#[cfg(feature = "jwt-param")]
#[cfg(not(all(feature = "jwt-header", feature = "jwt-query", feature = "jwt-cookie")))]
use viz_core::types::ParamsContextExt;

#[cfg(feature = "jwt-cookie")]
#[cfg(not(all(feature = "jwt-header", feature = "jwt-query", feature = "jwt-param")))]
use viz_core::types::CookieContextExt;

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::de::DeserializeOwned;

pub use jsonwebtoken;

/// JWT Middleware
#[derive(Debug)]
pub struct JWTMiddleware<T>
where
    T: Debug
{
    #[cfg(not(feature = "jwt-header"))]
    #[cfg(any(feature = "jwt-query", feature = "jwt-param", feature = "jwt-cookie"))]
    n: String,
    s: String,
    v: Validation,
    t: PhantomData<T>,
}

impl<T> JWTMiddleware<T>
where
    T: DeserializeOwned + Sync + Send + 'static + Debug,
{
    /// Creates JWT
    pub fn new() -> Self {
        Self {
            #[cfg(not(feature = "jwt-header"))]
            #[cfg(any(feature = "jwt-query", feature = "jwt-param", feature = "jwt-cookie"))]
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
    #[cfg(not(feature = "jwt-header"))]
    #[cfg(any(feature = "jwt-query", feature = "jwt-param", feature = "jwt-cookie"))]
    pub fn name(mut self, name: &str) -> Self {
        self.n = name.to_owned();
        self
    }

    #[tracing::instrument(skip(cx))]
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
                cx.cookie(&self.n).map(|c| c.to_string())
            } else {
                None
            }
        }
    }
}

impl<'a, T> Middleware<'a, Context> for JWTMiddleware<T>
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
