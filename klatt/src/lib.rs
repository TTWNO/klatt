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

#![no_std]
#![deny(clippy::cargo, clippy::pedantic, unsafe_code, rustdoc::all)]
// fine for us since loss of precision/sign is not that imporatnt, as long as it's the same every time.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

extern crate alloc;

mod klatt;
pub use klatt::{
    FrameParms, GlottalSourceType, MainParms, generate_sound,
    get_vocal_tract_transfer_function_coefficients,
};
mod poly_real;
