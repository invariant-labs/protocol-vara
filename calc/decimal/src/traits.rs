use core::fmt::Debug;

use alloc::string::String;

// custom traits need to be defined in order to implement the conversion on sails defined types
pub trait UintCast<T>: Sized {
    fn uint_cast(value: T) -> Self;
}

pub trait UintCheckedCast<T>: Sized {
    fn uint_checked_cast(value: T) -> Result<Self, String>;
}

impl<T> UintCast<T> for T {
    fn uint_cast(value: Self) -> Self {
        value
    }
}

pub trait Decimal: Sized {
    type U: Debug + Default;

    fn get(&self) -> Self::U;
    fn new(value: Self::U) -> Self;
    fn max_instance() -> Self;
    fn max_value() -> Self::U;
    fn here<Y: TryFrom<Self::U>>(&self) -> Y;
    fn scale() -> u8;
    fn checked_one() -> Result<Self, String>;
    fn one() -> Self;
    fn checked_almost_one() -> Result<Self, String>;
    fn almost_one() -> Self;
}

pub trait Conversion: Decimal {
    fn cast<T>(self) -> T
    where
        T: UintCast<<Self as Decimal>::U>,
    {
        T::uint_cast(self.get())
    }
    fn checked_cast<T>(self) -> Result<T, String>
    where
        T: UintCheckedCast<<Self as Decimal>::U>,
    {
        T::uint_checked_cast(self.get())
    }
    fn from_value<T, R>(from: R) -> T
    where
        T: UintCast<R>,
    {
        T::uint_cast(from)
    }
    fn checked_from_value<T, R>(from: R) -> Result<T, String>
    where
        T: UintCheckedCast<R>,
    {
        T::uint_checked_cast(from)
    }
}

pub trait BigOps<T>: Sized {
    fn big_mul(self, rhs: T) -> Self;
    fn big_mul_up(self, rhs: T) -> Self;
    fn big_div(self, rhs: T) -> Self;
    fn checked_big_div(self, rhs: T) -> Result<Self, String>;
    fn big_div_up(self, rhs: T) -> Self;
}

pub trait Others<T> {
    fn mul_up(self, rhs: T) -> Self;
    fn div_up(self, rhs: T) -> Self;
}

pub trait OthersSameType {
    fn sub_abs(self, rhs: Self) -> Self;
}

pub trait Factories<T>: Sized {
    fn from_integer(integer: T) -> Self;
    fn from_scale(integer: T, scale: u8) -> Self;
    fn checked_from_scale(integer: T, scale: u8) -> Result<Self, String>;
    fn from_scale_up(integer: T, scale: u8) -> Self;
}

pub trait FactoriesUnderlying {
    type U: Debug + Default;

    fn from_integer_underlying(integer: Self::U) -> Self;
    fn from_scale_underlying(integer: Self::U, scale: u8) -> Self;
    fn checked_from_scale_underlying(integer: Self::U, scale: u8) -> Result<Self, String>
    where
        Self: Sized;
    fn from_scale_up_underlying(integer: Self::U, scale: u8) -> Self;
}

pub trait BetweenDecimals<T>: Sized {
    fn from_decimal(other: T) -> Self;
    fn checked_from_decimal(other: T) -> Result<Self, String>;
    fn from_decimal_up(other: T) -> Self;
}

pub trait ToValue<T, B> {
    fn big_mul_to_value(self, value: T) -> B;
    fn big_mul_to_value_up(self, value: T) -> B;
}

pub trait FactoriesToValue<T, B> {
    fn checked_from_scale_to_value(integer: T, scale: u8) -> Result<B, String>;
}

pub trait BetweenDecimalsToValue<T, B> {
    fn checked_from_decimal_to_value(other: T) -> Result<B, String>;
}

pub trait ByNumber<B>: Sized {
    fn big_div_by_number(self, number: B) -> Self;
    fn big_div_by_number_up(self, number: B) -> Self;
    fn checked_big_div_by_number(self, number: B) -> Result<Self, String>;
    fn checked_big_div_by_number_up(self, number: B) -> Result<Self, String>;
}

pub trait CheckedOps: Sized {
    fn checked_add(self, rhs: Self) -> Result<Self, String>;
    fn checked_sub(self, rhs: Self) -> Result<Self, String>;
    fn checked_div(self, rhs: Self) -> Result<Self, String>;
}
