//! Basic math functions for T-Lang.

/// Absolute value of an integer.
pub fn abs(x: i64) -> i64 {
    x.abs()
}

/// Integer power: `base.pow(exp as u32)`
pub fn pow(base: i64, exp: u32) -> i64 {
    base.pow(exp)
}

/// Square root of a floating point number
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

/// Raise a float to a float power
pub fn powf(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

/// Natural logarithm
pub fn ln(x: f64) -> f64 {
    x.ln()
}

/// Base 10 logarithm
pub fn log10(x: f64) -> f64 {
    x.log10()
}

/// Sine function
pub fn sin(x: f64) -> f64 {
    x.sin()
}

/// Cosine function
pub fn cos(x: f64) -> f64 {
    x.cos()
}

/// Tangent function
pub fn tan(x: f64) -> f64 {
    x.tan()
}

/// Mathematical constants
pub const PI: f64 = std::f64::consts::PI;
pub const E: f64 = std::f64::consts::E;

/// Maximum of two values
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

/// Minimum of two values
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a <= b { a } else { b }
}

/// Clamp a value between min and max
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}