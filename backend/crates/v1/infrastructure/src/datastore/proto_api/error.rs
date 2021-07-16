use std::env;
use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unexpected status from GCP: {0}")]
    Status(#[from] tonic::Status),

    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] json::Error),

    #[error("environment error: {0}")]
    Env(#[from] env::VarError),

    #[cfg(feature = "storage")]
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("conversion error: {0}")]
    Convert(#[from] ConvertError),

    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),
}

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("expected property `{0}` was missing")]
    MissingProperty(String),

    #[error("expected property type `{expected}`, got `{got}`")]
    UnexpectedPropertyType { expected: String, got: String },
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("JWT error: {0}")]
    Jwt(#[from] jwt::errors::Error),

    #[error("JSON error: {0}")]
    Json(#[from] json::Error),

    #[error("Hyper error: {0}")]
    Http(#[from] http::Error),

    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
}
