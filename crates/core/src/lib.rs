mod traits;
mod types;

pub use baxe_derive::error;
pub use traits::BackendError;



#[test]
fn test_macro_generated_enum() {
    use baxe_derive::error;
    use axum::{http::StatusCode, response::IntoResponse, Json};
    use traits::BackendError;
    use serde::Serialize;

    #[derive(Debug, Default, Clone, Serialize, PartialEq)]
    pub enum Tags {
        #[default]
        Unknown,
        NotFound,
        BadRequest,
        SpecificDomainError
    }

    impl std::fmt::Display for Tags {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
            match *self {
                Tags::NotFound => write!(f, "NOT_FOUND"),
                Tags::BadRequest => write!(f, "BAD_REQUEST"),
                Tags::SpecificDomainError => write!(f, "SPECIFIC_DOMAIN_ERROR"),
                Tags::Unknown => write!(f, "UNKNOWN"),
            }
         }
    }

    impl std::str::FromStr for Tags {
        type Err = String;
    
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "NOT_FOUND" => Ok(Tags::NotFound),
                "BAD_REQUEST" => Ok(Tags::BadRequest),
                "SPECIFIC_DOMAIN_ERROR" => Ok(Tags::SpecificDomainError),
                _ => Err("Invalid tag".into())
            }
        }
    }

    baxe_error!(Tags, serde(rename_all = "camelCase"), derive(Clone));

    #[derive(Debug, thiserror::Error)]
    pub enum DomainError {
        #[error("This is the error message with argument: {0}")]
        SpecificDomainError(String)
    }

    #[error]
    enum AppError {
        #[baxe(status = StatusCode::NOT_FOUND, tag = Tags::NotFound, code = 404, message = "Resource not found")]
        NotFound,
        #[baxe(status = StatusCode::BAD_REQUEST, tag = Tags::BadRequest, code = 4000, message = "Bad request data: {0}, requests {1:?}")]
        BadRequest(String, Vec<usize>),
        #[baxe(status = StatusCode::INTERNAL_SERVER_ERROR, tag = Tags::SpecificDomainError, code = 500, message = "thiserror works too: {0}")]
        ThisError(DomainError),
        
    }

    let not_found = AppError::NotFound;
    assert_eq!(not_found.to_status_code(), StatusCode::NOT_FOUND);
    assert_eq!(format!("{}", not_found.to_error_tag()), "NOT_FOUND");
    assert_eq!(not_found.to_error_code(), 404);
    assert_eq!(not_found.to_string(), "Resource not found");

    let bad_request = AppError::BadRequest("Invalid input".into(), vec![32450, 56765]);
    assert_eq!(bad_request.to_status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(format!("{}", bad_request.to_error_tag()), "BAD_REQUEST");
    assert_eq!(bad_request.to_error_code(), 4000);
    assert_eq!(bad_request.to_string(), "Bad request data: Invalid input, requests [32450, 56765]");

    let this_error = AppError::ThisError(DomainError::SpecificDomainError("Core melted".into()));
    assert_eq!(this_error.to_status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(format!("{}", this_error.to_error_tag()), "SPECIFIC_DOMAIN_ERROR");
    assert_eq!(this_error.to_error_code(), 500);
    assert_eq!(this_error.to_string(), "thiserror works too: This is the error message with argument: Core melted");

    // Test conversion to BaxeError
    let baxe_error: BaxeError = not_found.into();
    assert_eq!(baxe_error.status_code, StatusCode::NOT_FOUND);
    assert_eq!(baxe_error.error_tag, Tags::NotFound);
    assert_eq!(baxe_error.code, 404);
    assert_eq!(baxe_error.message, Some("Resource not found".into()));

}