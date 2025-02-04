use alloc::vec::Vec;

pub trait Filter {
    fn set_passthrough(&mut self);
    fn set_mute(&mut self);
}

pub trait BasicFilter {
    /// Returns the polynomial coefficients of the filter transfer function in the z-plane.
    /// The returned array contains the top and bottom coefficients of the rational fraction, ordered in ascending powers.
    fn get_transfer_function_coefficients(&self) -> Vec<Vec<f64>>;
    /// Perform one step of a filter.
    fn step(&mut self, x: f64) -> f64;
}
