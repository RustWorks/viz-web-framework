//! JSON Web Token Middleware

use std::{future::Future, marker::PhantomData, pin::Pin};

use viz_core::{
    http::{
        header::{HeaderValue, WWW_AUTHENTICATE},
        StatusCode,
    },
    Context, Middleware, Response, Result,
};

#[cfg(feature = "jwt-header")]
use viz_core::http::headers::{
    authorization::{Authorization, Bearer},
    HeaderMapExt,
};

#[cfg(feature = "jwt-query")]
use std::collections::HashMap;
#[cfg(feature = "jwt-query")]
use viz_core::types::QueryContextExt;

#[cfg(feature = "jwt-param")]
use viz_core::types::ParamsContextExt;

#[cfg(feature = "jwt-cookie")]
use viz_core::types::CookieContextExt;

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::de::DeserializeOwned;

pub use jsonwebtoken;

/// JWT Middleware
#[derive(Debug)]
pub struct JWTMiddleware<T> {
    #[cfg(any(feature = "jwt-query", feature = "jwt-param", feature = "jwt-cookie"))]
    n: String,
    s: String,
    v: Validation,
    t: PhantomData<T>,
}

impl<T> JWTMiddleware<T>
where
    T: DeserializeOwned + Sync + Send + 'static,
{
    /// Creates JWT
    pub fn new() -> Self {
        Self {
            s: "secret".to_owned(),
            v: Validation::default(),
            t: PhantomData::default(),
            #[cfg(any(feature = "jwt-query", feature = "jwt-param", feature = "jwt-cookie"))]
            n: "token".to_owned(),
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
    #[cfg(any(feature = "jwt-query", feature = "jwt-param", feature = "jwt-cookie"))]
    pub fn name(mut self, name: &str) -> Self {
        self.n = name.to_owned();
        self
    }

    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let reason = if let Some(val) = self.get(cx) {
            match decode::<T>(&val, &DecodingKey::from_secret(self.s.as_ref()), &self.v) {
                Ok(token) => {
                    cx.extensions_mut().insert(token);
                    return cx.next().await;
                }
                Err(e) => e.to_string(),
            }
        } else {
            "Missing Token".to_string()
        };

        let mut res: Response = StatusCode::UNAUTHORIZED.into();
        res.headers_mut().insert(
            WWW_AUTHENTICATE,
            HeaderValue::from_str(&format!("JWT {}", reason))?,
        );
        Ok(res)
    }

    #[cfg(feature = "jwt-header")]
    fn get(&self, cx: &mut Context) -> Option<String> {
        cx.headers()
            .typed_get::<Authorization<Bearer>>()
            .map(|auth| auth.0.token().to_owned())
    }

    #[cfg(feature = "jwt-query")]
    fn get(&self, cx: &mut Context) -> Option<String> {
        cx.query::<HashMap<String, String>>()
            .ok()?
            .get(&self.n)
            .cloned()
    }

    #[cfg(feature = "jwt-param")]
    fn get(&self, cx: &mut Context) -> Option<String> {
        cx.param(&self.n).ok()
    }

    #[cfg(feature = "jwt-cookie")]
    fn get(&self, cx: &mut Context) -> Option<String> {
        cx.cookie(&self.n).map(|c| c.to_string())
    }
}

impl<'a, T> Middleware<'a, Context> for JWTMiddleware<T>
where
    T: DeserializeOwned + Sync + Send + 'static,
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
