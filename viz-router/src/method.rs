use viz_core::http;

/// Method
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Method {
    /// Any Verbs
    Any,
    /// Single Verb
    Verb(http::Method),
}

impl Method {
    /// Star
    pub const STAR: &'static str = "*";

    /// To str
    pub fn as_str(&self) -> &str {
        match self {
            Method::Any => Self::STAR,
            Method::Verb(method) => method.as_str(),
        }
    }
}

impl From<http::Method> for Method {
    fn from(method: http::Method) -> Self {
        Method::Verb(method)
    }
}

impl std::fmt::Debug for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
