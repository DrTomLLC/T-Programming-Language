//! Basic math functions for T-Lang.
//! These wrap Rust’s primitive operations; replace with custom code as needed.

/// Absolute value of an integer.
pub fn abs(x: i64) -> i64 {
    if x < 0 { -x } else { x }
}

/// Integer power: `base.pow(exp as u32)`
/// (panics if exp is negative or too large)
pub fn pow(base: i64, exp: u32) -> i64 {
    base.pow(exp)
}

/// Placeholder for more advanced functions (sqrt, sin, cos, etc.).
pub fn sqrt(_x: f64) -> f64 {
    unimplemented!("sqrt is not implemented yet")
}
