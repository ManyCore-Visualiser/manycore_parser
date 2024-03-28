use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ManycoreErrorKind {
    InfoError(&'static str),
    GenerationError(String),
    RoutingError(String),
}

#[derive(Debug)]
pub struct ManycoreError {
    error_kind: ManycoreErrorKind,
}

impl ManycoreError {
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
