## Error handling

The `Result<Response>` is expanded into `Result<Response, Error>`.

### General Error

HTTP StatusCode is `500` while responding `Error`

### Custom Error to respond

#### 4xx or 5xx - Viz

* `how!(err)`: Generates an error

* `reject!(err)`: Returns an error to response

```rust
use viz_utils::thiserror::Error as ThisError;
use viz_core::{http, Response};

#[derive(ThisError, Debug)]
enum UserError {
    #[error("User Not Found")]
    NotFound,
}

#[must_use]
impl Into<Response> for UserError {
    fn into(self) -> Response {
        (http::StatusCode::NOT_FOUND, self.to_string()).into()
    }
}

// 1
async fn custom_error_1() -> Result<Response> {
    // Err(Into::<Error>::into(Into::<Response>::into(UserError::NotFound)))
    Err(how!(UserError::NotFound))
}

// 2
async fn custom_error_2() -> Result<Response> {
    // return Err(Into::<Error>::into(Into::<Response>::into(UserError::NotFound)))
    reject!(UserError::NotFound)
}

// 3
async fn custom_error_3() -> Result<Response, UserError> {
    Err(UserError::NotFound)
}
```

#### 500 - anyhow

* `anyhow!(err)`: Generates an error

* `bail!(err)`: Returns an error to response

* `ensure!(expr, err)`: Returns an error with a condition to response

```rust
use viz_utils::thiserror::Error as ThisError;
use viz_core::{http, Response};

#[derive(ThisError, Debug)]
enum UserError {
    #[error("User Not Found")]
    NotFound,
}

// 1
async fn custom_error_1() -> Result<Response> {
    Err(anyhow!(UserError::NotFound))
}

// 2
async fn custom_error_2() -> Result<Response> {
    Err(UserError::NotFound.into())
}

// 3
async fn custom_error_3() -> Result<Response> {
    // return Err(anyhow!(UserError::NotFound))
    bail!(UserError::NotFound)
}

// 4
async fn custom_error_4() -> Result<Response> {
    // if !false { return Err(anyhow!(UserError::NotFound)); }
    ensure!(false, UserError::NotFound);
}
```
