use axum::{
    extract::rejection::JsonRejection,
    http::{
        StatusCode,
        header::{InvalidHeaderName, InvalidHeaderValue},
        method::InvalidMethod,
    },
    response::{IntoResponse, Response},
};
use color_eyre::eyre;
use tracing::error;

pub type Result<T, E = Error> = color_eyre::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to load configuration: {0}")]
    Config(#[from] shipwright_config::Error),
    /// Unauthenticated user
    ///
    /// Return a `401 Unauthorized` response on an invalid register token.
    #[error("invalid register token")]
    InvalidRegisterToken,
    /// Unauthenticated user
    ///
    /// Return a `401 Unauthorized` response on an unauthenticated user.
    #[error("unauthenticated user")]
    Unauthenticated,
    /// Could not render template
    ///
    /// Return `500 Internal Server Error` on a template rendering error.
    #[error("could not render template")]
    ViewEngine(#[from] shipwright_ui::Error),
    /// An error occured while interacting with the database.
    ///
    /// Return `500 Internal Server Error` on a db error.
    #[error("an error occured while interacting with the database")]
    Database(#[from] shipwright_db::Error),
    /// An error occured while sending an email.
    ///
    /// Return `500 Internal Server Error` on a mailer error.
    #[error("an error occured while sending an email")]
    Mailer(#[from] shipwright_mailer::Error),
    /// An error occured while interacting with worker storage.
    ///
    /// Return `500 Internal Server Error` on a worker storage error.
    #[error("error interacting with worker storage")]
    Worker(#[from] shipwright_worker::Error),

    #[error(transparent)]
    Http(#[from] axum::http::Error),

    #[error(transparent)]
    JSON(#[from] serde_json::Error),

    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error(transparent)]
    InvalidHeaderName(#[from] InvalidHeaderName),

    #[error(transparent)]
    InvalidMethod(#[from] InvalidMethod),
    /// Enumerate any possible app arrors here.
    ///
    /// Return `500 Internal Server Error` on a `eyre::Error`.
    #[error("Error: {0}")]
    Unexpected(#[from] eyre::Error),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Unauthenticated | Error::InvalidRegisterToken => StatusCode::UNAUTHORIZED,
            Error::ViewEngine(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Database(shipwright_db::Error::NoRecordFound) => StatusCode::NOT_FOUND,
            Error::Database(shipwright_db::Error::UniqueConstraint(_)) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Error::Database(shipwright_db::Error::ValidationError(_)) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Error::Database(shipwright_db::Error::DatabaseError(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Error::Database(shipwright_db::Error::PasswordHashError(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Error::Mailer(shipwright_mailer::Error::Request(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Error::Mailer(shipwright_mailer::Error::Validation(_)) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Error::Worker(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Http(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::JSON(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::JsonRejection(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InvalidHeaderValue(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InvalidHeaderName(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InvalidMethod(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::InvalidRegisterToken => {
                // TODO: Return a invalid register token view here.
                return (self.status_code(), "invalid register token".to_string()).into_response();
            }
            Error::Unauthenticated => {
                // TODO: Return a not authenticated view here.
                return (self.status_code(), "unauthenticated".to_string()).into_response();
            }
            Error::ViewEngine(ref err) => {
                // TODO: Return a not found view here.
                error!("an error occured while rendering a template: {:?}", err);
                return (self.status_code(), err.to_string()).into_response();
            }

            Error::Database(shipwright_db::Error::NoRecordFound) => {
                // TODO: Return a not found view here.

                return (self.status_code(), "no record found".to_string()).into_response();
            }
            Error::Database(shipwright_db::Error::UniqueConstraint(ref _err)) => {
                // TODO: Return a unique constaint error view here.
            }
            Error::Database(shipwright_db::Error::ValidationError(ref err)) => {
                // TODO: Return a validation error view here.
                return (self.status_code(), err.to_string()).into_response();
            }
            Error::Database(shipwright_db::Error::DatabaseError(ref err)) => {
                error!(
                    "an error occured while interacting with the database: {:?}",
                    err
                );
                return (self.status_code(), err.to_string()).into_response();
            }
            Error::Database(shipwright_db::Error::PasswordHashError(ref err)) => {
                // TODO: Return a password hash error view here.
                error!("an error occured while hashing a password: {:?}", err);
            }
            Error::Mailer(shipwright_mailer::Error::Request(ref err)) => {
                error!("an error occured while sending email request: {:?}", err);
            }
            Error::Mailer(shipwright_mailer::Error::Validation(ref err)) => {
                error!("invalid inputs to mailer: {:?}", err);
            }

            Error::Worker(ref err) => {
                error!("an error occured while interacting with worker: {:?}", err);
            }

            Error::Http(ref err) => {
                error!("an error occured while interacting with http: {:?}", err);
            }

            Error::JSON(ref err) => {
                error!("an error occured while parsing json: {:?}", err);
            }

            Error::JsonRejection(ref err) => {
                error!("an error occured while parsing json: {:?}", err);
            }

            Error::InvalidHeaderValue(ref err) => {
                error!("an error occured while parsing header value: {:?}", err);
            }

            Error::InvalidHeaderName(ref err) => {
                error!("an error occured while parsing header name: {:?}", err);
            }

            Error::InvalidMethod(ref err) => {
                error!("an error occured while parsing method: {:?}", err);
            }

            Error::Config(ref err) => {
                error!("an error occured while loading configuration: {:?}", err);
            }

            Error::Unexpected(ref err) => {
                error!("an internal server error occured: {:?}", err);
            }
        }

        // TODO: Return a default error view here.
        self.status_code().into_response()
    }
}
