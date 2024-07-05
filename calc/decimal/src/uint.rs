//! Large uint types

// required for clippy
#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::manual_range_contains)]
#[cfg(not(feature = "invariant-wasm"))]
pub use sails_rtl::{U128, U256, U512};
use uint::construct_uint;

construct_uint! {
    pub struct U448T(7);
}

construct_uint! {
    pub struct U384T(6);
}

construct_uint! {
    pub struct U320(5);
}

construct_uint! {
    pub struct U192T(3);
}

impl From<U384T> for U448T {
    fn from(n: U384T) -> Self {
        U448T([n.0[0], n.0[1], n.0[2], n.0[3], n.0[4], n.0[5], 0])
    }
}

#[cfg(feature = "invariant-wasm")]
pub mod invariant_wasm {
    use alloc::string::{String, ToString};
    use serde::{Deserialize, Serialize};
    use uint::construct_uint;
    construct_uint! {
        pub struct U128(2);
    }
    construct_uint! {
        pub struct U256(4);
    }
    construct_uint! {
        pub struct U512(8);
    }
    impl Serialize for U128 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for U128 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(Self::from_dec_str(&s).unwrap())
        }
    }
    impl Serialize for U256 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for U256 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(Self::from_dec_str(&s).unwrap())
        }
    }
}
#[cfg(feature = "invariant-wasm")]
pub use invariant_wasm::*;

#[allow(dead_code)]
pub fn checked_u320_to_u256(n: U320) -> Option<U256> {
    if !(n >> 256).is_zero() {
        return None;
    }

    Some(U256([
        n.low_u64(),
        (n >> 64).low_u64(),
        (n >> 128).low_u64(),
        (n >> 192).low_u64(),
    ]))
}

#[allow(dead_code)]
pub fn u320_to_u256(n: U320) -> U256 {
    checked_u320_to_u256(n).unwrap()
}

#[allow(dead_code)]
pub const fn to_u256(n: u128) -> U256 {
    U256([n as u64, (n >> 64) as u64, 0, 0])
}

#[allow(dead_code)]
pub fn u256_to_u320(n: U256) -> U320 {
    U320([
        n.low_u64(),
        (n >> 64).low_u64(),
        (n >> 128).low_u64(),
        (n >> 192).low_u64(),
        0,
    ])
}

#[allow(dead_code)]
pub fn to_u320(n: u128) -> U320 {
    u256_to_u320(to_u256(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_u256() {
        {
            let from = 0;
            let result = to_u256(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = 1;
            let result = to_u256(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = 1324342342433342342;
            let result = to_u256(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = u64::MAX as u128;
            let result = to_u256(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = u64::MAX as u128 + 1;
            let result = to_u256(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = u64::MAX as u128 + 2;
            let result = to_u256(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = u128::MAX;
            let result = to_u256(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
    }

    #[test]
    fn test_to_u320() {
        {
            let from = 0;
            let result = to_u320(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = 1;
            let result = to_u320(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = 1324342342433342342;
            let result = to_u320(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = u64::MAX as u128;
            let result = to_u320(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = u64::MAX as u128 + 1;
            let result = to_u320(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = u64::MAX as u128 + 2;
            let result = to_u320(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
        {
            let from = u128::MAX;
            let result = to_u320(from);
            let back = result.as_u128();
            assert_eq!(from, back);
        }
    }

    #[test]
    fn test_u320_methods() {
        let _max = U320::MAX;
        let _from = U320::from(10);
        let zero = U320::zero();
        let is_zero = zero.is_zero();
        assert!(is_zero);
    }

    #[test]
    fn test_u320_to_u256() {
        let max_u256 = U320([u64::MAX, u64::MAX, u64::MAX, u64::MAX, 0]);
        let max_u320 = U320([u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX]);
        // sample example
        {
            let u320: U320 = U320::from_dec_str("456974").unwrap();
            let u256: U256 = u320_to_u256(u320);
            assert_eq!(u256, U256::from_dec_str("456974").unwrap());
        }

        // max value fits into U256
        {
            let u320: U320 = max_u256.clone();
            let u256: U256 = u320_to_u256(u320);
            assert_eq!(u320, U320::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap());
            assert_eq!(u256, U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap());
        }
        // max value + 1 does not fit into U256
        {
            let u320: U320 = max_u256.clone() + 1;
            let u256: Option<U256> = checked_u320_to_u256(u320);
            assert_eq!(u256, None);
        }
        // max u320 value
        {
            let u320: U320 = max_u320.clone();
            let u256: Option<U256> = checked_u320_to_u256(u320);
            assert_eq!(
                u320,
                U320::from_dec_str("2135987035920910082395021706169552114602704522356652769947041607822219725780640550022962086936575").unwrap()
            );
            assert_eq!(u256, None);
        }
    }
}
