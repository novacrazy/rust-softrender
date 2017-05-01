//! Relatively simple software rendering of 3D meshes.

#![deny(missing_docs)]

extern crate nalgebra;
extern crate rayon;

#[cfg(feature = "image_compat")]
extern crate image;

pub mod utils;
pub mod pixel;
pub mod mesh;
pub mod render;

#[cfg(feature = "image_compat")]
pub mod image_compat;