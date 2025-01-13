mod traits;
mod types;


pub use baxe_derive::error;
pub use traits::BackendError;
pub use types::BaxeError;



#[test]
fn test_macro_generated_enum() {
    use baxe_derive::error;
    use axum::{http::StatusCode, response::IntoResponse, Json};
    use crate::traits::BackendError;


    #[derive(Debug, thiserror::Error)]
    pub enum DomainError {
        #[error("This is the error message with argument: {0}")]
        SpecificDomainError(String)
    }

    #[error]
    #[derive(Debug)]
    enum AppError {
        #[baxe(status = StatusCode::NOT_FOUND, tag = "NOT_FOUND", code = 404, message = "Resource not found")]
        NotFound,
        #[baxe(status = StatusCode::INTERNAL_SERVER_ERROR, tag = "SERVER_ERROR", code = 5000, message = "Internal server error")]
        InternalServerError,
        #[baxe(status = StatusCode::BAD_REQUEST, tag = "BAD_REQUEST", code = 4000, message = "Bad request data: {0}, requests {1:?}")]
        BadRequest(String, Vec<usize>),
        #[baxe(status = StatusCode::INTERNAL_SERVER_ERROR, tag = "DOMAIN_SPECIFIC_ERROR", code = 500, message = "thiserror works too: {0}")]
        ThisError(DomainError),
        
    }

    // Test trait implementations
    let not_found = AppError::NotFound;
    assert_eq!(not_found.to_status_code(), StatusCode::NOT_FOUND);
    assert_eq!(not_found.to_error_tag(), "NOT_FOUND");
    assert_eq!(not_found.to_error_code(), 404);
    assert_eq!(not_found.to_string(), "Resource not found");

    let bad_request = AppError::BadRequest("Invalid input".into(), vec![32450, 56765]);
    assert_eq!(bad_request.to_status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(bad_request.to_error_tag(), "BAD_REQUEST");
    assert_eq!(bad_request.to_error_code(), 4000);
    assert_eq!(bad_request.to_string(), "Bad request data: Invalid input, requests [32450, 56765]");

    let this_error = AppError::ThisError(DomainError::SpecificDomainError("Core melted".into()));
    assert_eq!(this_error.to_status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(this_error.to_error_tag(), "DOMAIN_SPECIFIC_ERROR");
    assert_eq!(this_error.to_error_code(), 500);
    assert_eq!(this_error.to_string(), "thiserror works too: This is the error message with argument: Core melted");

    // Test conversion to BaxeError
    let baxe_error: crate::BaxeError = not_found.into();
    assert_eq!(baxe_error.status_code, StatusCode::NOT_FOUND);
    assert_eq!(baxe_error.error_tag, "NOT_FOUND");
    assert_eq!(baxe_error.code, 404);
    assert_eq!(baxe_error.message, "Resource not found");

}