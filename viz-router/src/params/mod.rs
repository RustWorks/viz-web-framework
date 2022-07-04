mod de;

use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use serde::de::DeserializeOwned;
use viz_core::{
    async_trait, Body, FromRequest, IntoResponse, Request, Response, StatusCode, ThisError,
};

pub(crate) use de::PathDeserializer;

#[derive(Debug)]
pub struct Params<T = Vec<(String, String)>>(pub T);

impl<T> AsRef<T> for Params<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Params<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Params<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl From<Vec<(&str, &str)>> for Params {
    fn from(v: Vec<(&str, &str)>) -> Self {
        Self(
            v.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }
}

impl Params {
    /// Gets single parameter by name.
    pub fn find<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.iter()
            .find(|p| p.0 == name)
            .ok_or_else(|| ParamsError::SingleParse(format!("missing {} param", name)))?
            .1
            .parse()
            .map_err(|e: T::Err| ParamsError::SingleParse(e.to_string()))
    }
}

#[async_trait]
impl<T> FromRequest for Params<T>
where
    T: DeserializeOwned,
{
    type Error = ParamsError;

    async fn extract(req: &mut Request<Body>) -> Result<Self, Self::Error> {
        req.params().map(Params)
    }
}

#[derive(ThisError, Debug)]
pub enum ParamsError {
    #[error("{}", .0)]
    SingleParse(String),
    #[error(transparent)]
    Parse(#[from] serde::de::value::Error),
    #[error("params is empty")]
    Empty,
}

impl IntoResponse for ParamsError {
    fn into_response(self) -> Response<Body> {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

pub trait ParamsRequestExt {
    /// Gets all parameters.
    fn params<T>(&self) -> Result<T, ParamsError>
    where
        T: DeserializeOwned;

    /// Gets single parameter by name.
    fn param<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: FromStr,
        T::Err: Display;
}

impl ParamsRequestExt for Request<Body> {
    fn params<T>(&self) -> Result<T, ParamsError>
    where
        T: DeserializeOwned,
    {
        match self.extensions().get::<Params>() {
            None => Err(ParamsError::Empty),
            Some(params) => {
                T::deserialize(PathDeserializer::new(&params)).map_err(ParamsError::Parse)
            }
        }
    }
    fn param<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.extensions()
            .get::<Params>()
            .ok_or(ParamsError::Empty)?
            .find(name)
    }
}
