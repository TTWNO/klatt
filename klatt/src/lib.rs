#![no_std]

extern crate alloc;

pub mod app_params;
pub use app_params::{f_params, m_parms};
pub mod klatt;
pub use klatt::{generate_sound, get_vocal_tract_transfer_function_coefficients};
pub mod poly_real;
