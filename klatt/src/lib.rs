#![no_std]
#![deny(clippy::cargo, clippy::pedantic)]

extern crate alloc;

mod klatt;
pub use klatt::{
    FrameParms, GlottalSourceType, MainParms, generate_sound,
    get_vocal_tract_transfer_function_coefficients,
};
mod poly_real;
