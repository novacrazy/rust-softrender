use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use trace_error::TraceResult;

pub type RenderResult<T> = TraceResult<T, RenderError>;

#[derive(Debug)]
pub enum RenderError {
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
