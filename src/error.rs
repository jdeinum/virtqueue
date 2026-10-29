//! This module contains the error types for the virtio implementation. I use the same approach as
//! `std::io` implementations purely for the sake of semver management.

use std::fmt::{Debug, Display};
use std::sync::Arc;
use thiserror::Error;

/// The kind of errors we may run into with a virtio queue, marked non-exhaustive so callers are
/// always expected to handle everything and I can freely handle errors.
#[derive(Copy, Clone, PartialEq, Debug, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    Unexpected,
    RequestTooLarge,
}

/// The representation of our error. This is private and callers can not access it, the only use if
/// printing it.
#[derive(Error, Debug)]
pub enum ErrorRepr {
    #[error("Request too large: {0}, not enough descriptors")]
    RequestTooLarge(usize),

    #[error("Unexpected error")]
    UnexpectedError(#[from] anyhow::Error),
}

/// Our virtio queue error. Because `ErrorRepr` is sized, this means our errors are always 8 bytes,
/// and the Arc allows cheap sharing.
#[derive(Clone)]
pub struct Error(Arc<ErrorRepr>);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&*self.0, f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match &*self.0 {
            ErrorRepr::UnexpectedError(_) => ErrorKind::Unexpected,
            ErrorRepr::RequestTooLarge(_) => ErrorKind::RequestTooLarge,
        }
    }
}

impl From<ErrorRepr> for Error {
    fn from(value: ErrorRepr) -> Self {
        Self(Arc::new(value))
    }
}

/// Allows us to use ? on anyhow errors, but they should be used liberally and more descriptive
/// errors should be used
impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        ErrorRepr::from(value).into()
    }
}
