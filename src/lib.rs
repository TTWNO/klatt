//! Klatt Formant Speech Synthesis in Rust.
//!
//! *NOTE*: This is _not_ a text-to-speech engine.
//! This is only synthesis from various parametric values.
//! See examples on how to use this.
//!
//! ## `no_std`
//!
//! This library is unconditionally `no_std` compatible.
//! `alloc` is required for now, but I'm looking into options for allowing `no_alloc`.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    clippy::all,
    clippy::cargo,
    clippy::pedantic,
    unsafe_code,
    rustdoc::all
)]
// fine for us since loss of precision/sign is not that imporatnt, as long as it's the same every time.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

#[cfg(all(feature = "std", feature = "libm"))]
compile_error!("Features \"std\" and \"libm\" are mutually exclusive.");

#[cfg(not(any(feature = "std", feature = "libm")))]
compile_error!("Must specify a math feature: either \"std\" or \"libm\".");

extern crate alloc;

mod traits;
pub use traits::{BasicFilter, Filter};
mod klatt;
mod math;
pub use klatt::{
    FrameParms, GlottalSourceType, MainParms, generate_sound,
    get_vocal_tract_transfer_function_coefficients,
};
mod poly_real;
