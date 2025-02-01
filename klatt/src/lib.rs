#![no_std]
#![deny(
    clippy::cargo,
)]

extern crate alloc;

mod klatt;
pub use klatt::{generate_sound, get_vocal_tract_transfer_function_coefficients, FrameParms, GlottalSourceType, MainParms};
mod poly_real;
