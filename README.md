# Better Axum Errors

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io Version](https://img.shields.io/crates/v/baxe)](https://crates.io/crates/baxe)

## Description

**Better Axum Error** (aka. _baxe_) is a utility that streamlines error handling in backend services built with Axum. With a simple and readable macro, you can define your backend errors once and automatically generate standardized JSON error responses, saving time and reducing complexity.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

## Installation

```
cargo add baxe
```

## Usage

### Attributes

 * `logMessageWith`: allows to use a logger to log the error message before sending the response
 * `hideMessage`: allows to exclude the message from the body of the response

### Example

```rust
use baxe::{baxe_error, BackendError}; 

baxe_error!(String, serde(rename_all = "camelCase"), derive(Clone));

// Optional: thiserror definitions work with baxe too 
#[derive(Debug, thiserror::Error)]
pub enum EmailValidationError {
    #[error("Email address syntax is invalid: received '{0}', expected value matching '{1}'")]
    InvalidSyntax(String, String),
    #[error("Email domain is unknown")]
    UnknownDomain
}


#[baxe::error(logMessageWith=tracing::error, hideMessage)] // Use the macro to define your errors
pub enum BackendErrors {
    #[baxe(status = StatusCode::BAD_REQUEST, tag = "bad_request", code = 400, message = "Bad request: {0}")]
    BadRequest(String),
    #[baxe(status = StatusCode::UNAUTHORIZED, tag = "auth/invalid_email_or_password", code = 10_000, message = "Invalid email or password")]
    InvalidEmailOrPassword,
    #[baxe(status = StatusCode::BAD_REQUEST, tag = "auth/invalid_email_format", code = 10_001, message = "Invalid email format: {0}")]
    InvalidEmailFormat(EmailValidationError),
}
```

Example axum handler:

```rust
pub async fn handler() -> Result<Json<String>, BaxeError> {
    if let Err(e) = validate_email(email) {
        return Err(BackendErrors::InvalidEmailFormat(e).into());
    }

    Ok(Json("Hello, world!".to_string()))
}
```

The above code allows to log a descriptive error:

```bash
2025-01-10T09:58:56.677274Z  ERROR my_app:handlers: Invalid email format: Email address syntax is invalid: received 'example.com', expected value matching '^[^@]+@[^@]+\.[^@]+$'"
```

and automatically generates the following error response type:

```rust
#[derive(std::fmt::Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone)]
pub struct BaxeError {
    #[serde(skip)]
    pub status_code: axum::http::StatusCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub code: u16,
    pub error_tag: String,
}
```

that is serialized to the following json response:

```json
{
  "code": 10001,
  "errorTag": "auth/invalid_email_format"
}
```

## Contributing

Feel free to open issues and send PRs. We will evaluate them together in the comment section.

## License

This project is licensed under the [MIT License](LICENSE).
