//! Error handling structures

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use trace_error::TraceResult;

/// Helpful `Result` type alias
pub type RenderResult<T> = TraceResult<T, RenderError>;

/// Errors that may occur during rendering or general usage of the library
#[derive(Debug)]
pub enum RenderError {
    /// An invalid coordinate was used to access a pixel
    InvalidPixelCoordinate
}

impl Display for RenderError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl Error for RenderError {
    fn description(&self) -> &str {
        match *self {
            RenderError::InvalidPixelCoordinate => "Invalid Pixel Coordinate"
        }
    }
}
