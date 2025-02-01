#![no_std]
#![deny(clippy::cargo, clippy::pedantic, unsafe_code)]
// fine for us since loss of precision/sign is not that imporatnt, as long as it's the same every time.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

extern crate alloc;

mod klatt;
pub use klatt::{
    FrameParms, GlottalSourceType, MainParms, generate_sound,
    get_vocal_tract_transfer_function_coefficients,
};
mod poly_real;
