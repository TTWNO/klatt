//! Core math functions for the synthesis.
//! If the `libm` feature is enabled, this just exports the required functions.
//! If the `std` feature is enabled, this converts the syntax from the std variety: `f.sqrt()` into
//! the `libm` equiv. `sqrt(f)`.

#[cfg(feature = "libm")]
pub(crate) use libm::{cos, exp, pow, round, sin, sqrt};

#[cfg(feature = "std")]
pub(crate) fn sqrt(f: f64) -> f64 {
    f.sqrt()
}
#[cfg(feature = "std")]
pub(crate) fn pow(f1: f64, f2: f64) -> f64 {
    f1.powf(f2)
}
#[cfg(feature = "std")]
pub(crate) fn cos(f: f64) -> f64 {
    f.cos()
}
#[cfg(feature = "std")]
pub(crate) fn sin(f: f64) -> f64 {
    f.cos()
}
#[cfg(feature = "std")]
pub(crate) fn exp(f: f64) -> f64 {
    f.exp()
}
#[cfg(feature = "std")]
pub(crate) fn round(f: f64) -> f64 {
    f.round()
}
