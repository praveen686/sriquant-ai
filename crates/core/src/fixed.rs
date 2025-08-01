//! Fixed-point arithmetic implementation
//! 
//! Provides a Fixed structure that supports numerical values up to 
//! 999999.999999999999, catering to the precision needs of financial calculations.

use rust_decimal::{Decimal, prelude::*};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign};
use std::str::FromStr;

/// Fixed-point decimal type for precise financial calculations
/// 
/// High-performance architecture, supports values up to 999999.999999999999
/// with exact decimal precision to avoid floating-point errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Fixed {
    value: Decimal,
}

impl Fixed {
    /// Maximum value: 999999.999999999999
    pub fn max() -> Self {
        Fixed {
            value: Decimal::new(999999999999999999i64, 12),
        }
    }
    
    /// Minimum value: -999999.999999999999  
    pub fn min() -> Self {
        Fixed {
            value: Decimal::new(-999999999999999999i64, 12),
        }
    }
    
    /// Zero value
    pub const ZERO: Fixed = Fixed {
        value: Decimal::ZERO,
    };
    
    /// One value
    pub const ONE: Fixed = Fixed {
        value: Decimal::ONE,
    };
    
    /// Create a new Fixed from a Decimal
    pub fn from_decimal(value: Decimal) -> Result<Self, FixedError> {
        let fixed = Fixed { value };
        
        if fixed > Self::max() || fixed < Self::min() {
            return Err(FixedError::OutOfRange);
        }
        
        Ok(fixed)
    }
    
    /// Create a Fixed from an integer
    pub fn from_i64(value: i64) -> Result<Self, FixedError> {
        Self::from_decimal(Decimal::from(value))
    }
    
    /// Create a Fixed from a float (use with caution)
    pub fn from_f64(value: f64) -> Result<Self, FixedError> {
        let decimal = Decimal::try_from(value)
            .map_err(|_| FixedError::InvalidValue)?;
        Self::from_decimal(decimal)
    }
    
    /// Create a Fixed from a string
    pub fn from_str_exact(s: &str) -> Result<Self, FixedError> {
        let decimal = Decimal::from_str(s)
            .map_err(|_| FixedError::InvalidValue)?;
        Self::from_decimal(decimal)
    }
    
    /// Get the underlying Decimal value
    pub fn to_decimal(&self) -> Decimal {
        self.value
    }
    
    /// Convert to f64 (may lose precision)
    pub fn to_f64(&self) -> f64 {
        self.value.to_f64().unwrap_or(0.0)
    }
    
    /// Convert to string with all decimal places
    pub fn to_string_exact(&self) -> String {
        self.value.to_string()
    }
    
    /// Convert to string with specified decimal places
    pub fn to_string_with_scale(&self, scale: u32) -> String {
        format!("{:.1$}", self.value, scale as usize)
    }
    
    /// Check if the value is zero
    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
    
    /// Check if the value is positive
    pub fn is_positive(&self) -> bool {
        self.value.is_sign_positive()
    }
    
    /// Check if the value is negative
    pub fn is_negative(&self) -> bool {
        self.value.is_sign_negative()
    }
    
    /// Get the absolute value
    pub fn abs(&self) -> Self {
        Fixed {
            value: self.value.abs(),
        }
    }
    
    /// Round to specified decimal places
    pub fn round_dp(&self, dp: u32) -> Self {
        Fixed {
            value: self.value.round_dp(dp),
        }
    }
    
    /// Truncate to specified decimal places
    pub fn trunc_with_scale(&self, scale: u32) -> Self {
        Fixed {
            value: self.value.trunc_with_scale(scale),
        }
    }
    
    /// Calculate percentage of another Fixed value
    pub fn percent_of(&self, other: Fixed) -> Result<Fixed, FixedError> {
        if other.is_zero() {
            return Err(FixedError::DivisionByZero);
        }
        
        let result = (*self / other) * Fixed::from_i64(100)?;
        Ok(result)
    }
    
    /// Apply percentage to this value
    pub fn apply_percent(&self, percent: Fixed) -> Result<Fixed, FixedError> {
        let multiplier = Fixed::ONE + (percent / Fixed::from_i64(100)?);
        Ok(*self * multiplier)
    }
}

/// Fixed-point arithmetic errors
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FixedError {
    #[error("Value out of range (max: 999999.999999999999)")]
    OutOfRange,
    #[error("Invalid value")]
    InvalidValue,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Overflow in arithmetic operation")]
    Overflow,
}

// Arithmetic implementations
impl Add for Fixed {
    type Output = Fixed;
    
    fn add(self, rhs: Self) -> Self::Output {
        Fixed {
            value: self.value + rhs.value,
        }
    }
}

impl Sub for Fixed {
    type Output = Fixed;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Fixed {
            value: self.value - rhs.value,
        }
    }
}

impl Mul for Fixed {
    type Output = Fixed;
    
    fn mul(self, rhs: Self) -> Self::Output {
        Fixed {
            value: self.value * rhs.value,
        }
    }
}

impl Div for Fixed {
    type Output = Fixed;
    
    fn div(self, rhs: Self) -> Self::Output {
        Fixed {
            value: self.value / rhs.value,
        }
    }
}

// Assignment operators
impl AddAssign for Fixed {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
    }
}

impl SubAssign for Fixed {
    fn sub_assign(&mut self, rhs: Self) {
        self.value -= rhs.value;
    }
}

impl MulAssign for Fixed {
    fn mul_assign(&mut self, rhs: Self) {
        self.value *= rhs.value;
    }
}

impl DivAssign for Fixed {
    fn div_assign(&mut self, rhs: Self) {
        self.value /= rhs.value;
    }
}

impl Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl FromStr for Fixed {
    type Err = FixedError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_exact(s)
    }
}

impl From<Decimal> for Fixed {
    fn from(value: Decimal) -> Self {
        Fixed { value }
    }
}

impl From<Fixed> for Decimal {
    fn from(fixed: Fixed) -> Self {
        fixed.value
    }
}

/// Convenience macro for creating Fixed values
#[macro_export]
macro_rules! fixed {
    ($value:expr) => {
        $crate::fixed::Fixed::from_str_exact(stringify!($value)).unwrap()
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fixed_creation() {
        let f1 = Fixed::from_str_exact("123.456").unwrap();
        assert_eq!(f1.to_string(), "123.456");
        
        let f2 = Fixed::from_i64(100).unwrap();
        assert_eq!(f2.to_string(), "100");
    }
    
    #[test]
    fn test_fixed_arithmetic() {
        let f1 = Fixed::from_str_exact("10.5").unwrap();
        let f2 = Fixed::from_str_exact("2.5").unwrap();
        
        assert_eq!((f1 + f2).to_string(), "13.0");
        assert_eq!((f1 - f2).to_string(), "8.0");
        assert_eq!((f1 * f2).to_string(), "26.25");
        assert_eq!((f1 / f2).to_string(), "4.20");
    }
    
    #[test]
    fn test_fixed_limits() {
        let max_result = Fixed::from_str_exact("999999.999999999999");
        assert!(max_result.is_ok());
        
        let over_max = Fixed::from_str_exact("1000000.0");
        assert!(over_max.is_err());
    }
    
    #[test]
    fn test_fixed_percentage() {
        let base = Fixed::from_str_exact("100.0").unwrap();
        let percent = Fixed::from_str_exact("10.0").unwrap();
        
        let result = base.apply_percent(percent).unwrap();
        assert_eq!(result.to_string(), "110.00");
    }
    
    #[test]
    fn test_fixed_macro() {
        let f = fixed!(123.456);
        assert_eq!(f.to_string(), "123.456");
    }
}