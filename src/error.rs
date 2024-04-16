use std::{error::Error, fmt::Display};

#[cfg(doc)]
use crate::ManycoreSystem;

/// Enum to wrap possible errors that might arise when generating/updating a [`ManycoreSystem`].
///
/// The string contained in each variant is a user friendly explanation of the error (or a call to `to_string()` on the error).
#[derive(Debug)]
pub enum ManycoreErrorKind {
    InfoError(&'static str),
    GenerationError(String),
    RoutingError(String),
}

/// A generic error container used to keep results consistent within the library.
#[derive(Debug)]
pub struct ManycoreError {
    error_kind: ManycoreErrorKind,
}

impl ManycoreError {
    /// Instantiates a new [`ManycoreError`] instance.
    pub fn new(error_kind: ManycoreErrorKind) -> Self {
        Self { error_kind }
    }
}

impl Display for ManycoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error_kind {
            ManycoreErrorKind::InfoError(e) => write!(f, "Info Error: {}", e),
            ManycoreErrorKind::GenerationError(e) => write!(f, "Generation Error: {}", e),
            ManycoreErrorKind::RoutingError(e) => write!(f, "Routing Error: {}", e),
        }
    }
}

impl Error for ManycoreError {}
