use viz_core::http;

#[derive(Eq, PartialEq, Hash)]
pub enum Method {
    All,
    Verb(http::Method),
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::All => "*",
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
