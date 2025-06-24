// File: tstd/src/math.rs - COMPLETE REWRITE
// -----------------------------------------------------------------------------

//! Complete mathematical functions for T-Lang standard library.

// No need to import std::f64::consts at the top level since we reference it directly in our consts module

/// Absolute value of a number
pub fn abs<T>(x: T) -> T
where
    T: PartialOrd + std::ops::Neg<Output = T> + Copy + Default
{
    if x < T::default() { -x } else { x }
}

/// Integer absolute value
pub fn abs_i32(x: i32) -> i32 { x.abs() }
pub fn abs_i64(x: i64) -> i64 { x.abs() }

/// Floating point absolute value
pub fn abs_f32(x: f32) -> f32 { x.abs() }
pub fn abs_f64(x: f64) -> f64 { x.abs() }

/// Square root
pub fn sqrt(x: f64) -> Result<f64, String> {
    if x < 0.0 {
        Err(format!("Square root of negative number: {}", x))
    } else if x.is_nan() {
        Err("Square root of NaN".to_string())
    } else {
        Ok(x.sqrt())
    }
}

/// Trigonometric functions
pub fn sin(x: f64) -> f64 { x.sin() }
pub fn cos(x: f64) -> f64 { x.cos() }
pub fn tan(x: f64) -> f64 { x.tan() }

/// Inverse trigonometric functions
pub fn asin(x: f64) -> Result<f64, String> {
    if x < -1.0 || x > 1.0 {
        Err(format!("asin domain error: {}", x))
    } else {
        Ok(x.asin())
    }
}

pub fn acos(x: f64) -> Result<f64, String> {
    if x < -1.0 || x > 1.0 {
        Err(format!("acos domain error: {}", x))
    } else {
        Ok(x.acos())
    }
}

pub fn atan(x: f64) -> f64 { x.atan() }
pub fn atan2(y: f64, x: f64) -> f64 { y.atan2(x) }

/// Hyperbolic functions
pub fn sinh(x: f64) -> f64 { x.sinh() }
pub fn cosh(x: f64) -> f64 { x.cosh() }
pub fn tanh(x: f64) -> f64 { x.tanh() }

/// Logarithmic functions
pub fn ln(x: f64) -> Result<f64, String> {
    if x <= 0.0 {
        Err(format!("Natural log of non-positive number: {}", x))
    } else {
        Ok(x.ln())
    }
}

pub fn log10(x: f64) -> Result<f64, String> {
    if x <= 0.0 {
        Err(format!("Log10 of non-positive number: {}", x))
    } else {
        Ok(x.log10())
    }
}

pub fn log2(x: f64) -> Result<f64, String> {
    if x <= 0.0 {
        Err(format!("Log2 of non-positive number: {}", x))
    } else {
        Ok(x.log2())
    }
}

/// Exponential functions
pub fn exp(x: f64) -> f64 { x.exp() }
pub fn exp2(x: f64) -> f64 { x.exp2() }
pub fn exp10(x: f64) -> f64 { 10.0_f64.powf(x) }

/// Power functions
pub fn pow(base: f64, exp: f64) -> f64 { base.powf(exp) }
pub fn powi(base: f64, exp: i32) -> f64 { base.powi(exp) }

/// Root functions
pub fn cbrt(x: f64) -> f64 { x.cbrt() }
pub fn hypot(x: f64, y: f64) -> f64 { x.hypot(y) }

/// Rounding functions
pub fn floor(x: f64) -> f64 { x.floor() }
pub fn ceil(x: f64) -> f64 { x.ceil() }
pub fn round(x: f64) -> f64 { x.round() }
pub fn trunc(x: f64) -> f64 { x.trunc() }

/// Min/max functions
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a <= b { a } else { b }
}

pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

/// Clamp function
pub fn clamp<T: PartialOrd>(value: T, min_val: T, max_val: T) -> T {
    if value < min_val {
        min_val
    } else if value > max_val {
        max_val
    } else {
        value
    }
}

/// Mathematical constants
pub mod consts {
    pub const PI: f64 = std::f64::consts::PI;
    pub const E: f64 = std::f64::consts::E;
    pub const TAU: f64 = std::f64::consts::TAU;
    pub const SQRT_2: f64 = std::f64::consts::SQRT_2;
    pub const SQRT_3: f64 = 1.7320508075688772;
    pub const LN_2: f64 = std::f64::consts::LN_2;
    pub const LN_10: f64 = std::f64::consts::LN_10;
    pub const LOG2_E: f64 = std::f64::consts::LOG2_E;
    pub const LOG10_E: f64 = std::f64::consts::LOG10_E;
    pub const FRAC_1_PI: f64 = std::f64::consts::FRAC_1_PI;
    pub const FRAC_2_PI: f64 = std::f64::consts::FRAC_2_PI;
    pub const FRAC_PI_2: f64 = std::f64::consts::FRAC_PI_2;
    pub const FRAC_PI_3: f64 = std::f64::consts::FRAC_PI_3;
    pub const FRAC_PI_4: f64 = std::f64::consts::FRAC_PI_4;
    pub const FRAC_PI_6: f64 = std::f64::consts::FRAC_PI_6;
    pub const FRAC_PI_8: f64 = std::f64::consts::FRAC_PI_8;
}

/// Numeric utilities
pub fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

pub fn lcm(a: u64, b: u64) -> u64 {
    (a * b) / gcd(a, b)
}
