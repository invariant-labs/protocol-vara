#![no_std]

extern crate alloc;

mod traits;
mod uint;

pub use crate::uint::{
    checked_u320_to_u256, to_u256, u256_to_u320, U128, U192T, U256, U320, U384T, U448T, U512,
};
pub use decimal_core::decimal;
pub use num_traits;
pub use traits::*;

#[cfg(test)]
pub mod tests {
    use super::*;
    use decimal_core::decimal;

    #[cfg(test)]
    #[decimal(3, U256)]
    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    struct R(U128);

    #[cfg(test)]
    #[decimal(1)]
    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    struct Q {
        v: U128,
    }

    #[cfg(test)]
    #[decimal(0)]
    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    struct N(U128);

    #[cfg(test)]
    #[decimal(1)]
    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    struct M(u128);

    #[test]
    fn test_cast() {
        assert_eq!(R::max_instance().cast::<U256>(), U256::from(u128::MAX));
        assert_eq!(R::max_instance().cast::<U192T>(), U192T::from(u128::MAX));

        assert_eq!(M::max_instance().cast::<U256>(), U256::from(u128::MAX));
        assert_eq!(M::max_instance().cast::<U192T>(), U192T::from(u128::MAX));

        assert_eq!(Q::max_instance().cast::<U256>(), U256::from(u128::MAX));
        assert_eq!(Q::max_instance().cast::<U192T>(), U192T::from(u128::MAX));

        assert_eq!(N::max_instance().cast::<U256>(), U256::from(u128::MAX));
        assert_eq!(N::max_instance().cast::<U192T>(), U192T::from(u128::MAX));
    }

    #[test]
    fn test_factories() {
        let _r = R::from_integer(U128::from(0));
        let _q = Q::from_integer(U128::from(0));
        let _n = N::from_integer(U128::from(0));
        let _m = M::from_integer(0u128);

        let _r = R::from_integer(0);
        let _q = Q::from_integer(0);
        let _n = N::from_integer(0);
        let _m = M::from_integer(0);
    }

    #[test]
    fn test_from_decimal() {
        let r = R(U128::from(42));
        let q = Q { v: U128::from(144) };
        let n = N(U128::from(3));
        let m = M::new(200000u128);

        assert_eq!(R::from_decimal(r), r);
        assert_eq!(R::from_decimal(q), R(U128::from(14400)));
        assert_eq!(R::from_decimal(n), R(U128::from(3000)));
        assert_eq!(R::from_decimal(m), R(U128::from(20000000)));

        assert_eq!(Q::from_decimal(r), Q { v: U128::from(0) });
        assert_eq!(Q::from_decimal(q), q);
        assert_eq!(Q::from_decimal(n), Q { v: U128::from(30) });
        assert_eq!(
            Q::from_decimal(m),
            Q {
                v: U128::from(200000)
            }
        );

        assert_eq!(N::from_decimal(r), N(U128::from(0)));
        assert_eq!(N::from_decimal(q), N(U128::from(14)));
        assert_eq!(N::from_decimal(n), n);
        assert_eq!(N::from_decimal(m), N(U128::from(20000)));

        assert_eq!(M::from_decimal(r), M(0));
        assert_eq!(M::from_decimal(q), M(144u128));
        assert_eq!(M::from_decimal(n), M(30u128));
        assert_eq!(M::from_decimal(m), m);
    }

    #[test]
    fn test_from_decimal_up() {
        let r = R(U128::from(42));
        let q = Q { v: U128::from(144) };
        let n = N(U128::from(3));
        let m = M::new(200000u128);

        assert_eq!(R::from_decimal_up(r), r);
        assert_eq!(R::from_decimal_up(q), R(U128::from(14400)));
        assert_eq!(R::from_decimal_up(n), R(U128::from(3000)));
        assert_eq!(R::from_decimal_up(m), R(U128::from(20000000)));

        assert_eq!(Q::from_decimal_up(r), Q { v: U128::from(1) });
        assert_eq!(Q::from_decimal_up(q), q);
        assert_eq!(Q::from_decimal_up(n), Q { v: U128::from(30) });
        assert_eq!(
            Q::from_decimal_up(m),
            Q {
                v: U128::from(200000)
            }
        );

        assert_eq!(N::from_decimal_up(r), N(U128::from(1)));
        assert_eq!(N::from_decimal_up(n), n);
        assert_eq!(N::from_decimal_up(q), N(U128::from(15)));
        assert_eq!(N::from_decimal_up(m), N(U128::from(20000)));

        assert_eq!(Q::from_decimal_up(r), Q { v: U128::from(1) });
        assert_eq!(Q::from_decimal_up(q), q);
        assert_eq!(Q::from_decimal_up(n), Q { v: U128::from(30) });
        assert_eq!(
            Q::from_decimal_up(m),
            Q {
                v: U128::from(200000)
            }
        );
    }

    #[test]
    fn test_ops() {
        assert_eq!(N(U128::from(0)) + N(U128::from(0)), N::new(U128::from(0)));
        assert_eq!(N(U128::from(1)) + N(U128::from(2)), N::new(U128::from(3)));
        assert_eq!(R(U128::from(0)) + R(U128::from(0)), R::new(U128::from(0)));
        assert_eq!(R(U128::from(1)) + R(U128::from(2)), R::new(U128::from(3)));
        assert_eq!(M::new(0) + M::new(0), M::new(0));
        assert_eq!(M::new(1) + M::new(2), M::new(3));

        assert_eq!(N(U128::from(0)) - N(U128::from(0)), N::new(U128::from(0)));
        assert_eq!(N(U128::from(2)) - N(U128::from(1)), N::new(U128::from(1)));
        assert_eq!(R(U128::from(0)) - R(U128::from(0)), R::new(U128::from(0)));
        assert_eq!(R(U128::from(2)) - R(U128::from(1)), R::new(U128::from(1)));
        assert_eq!(M::new(0) - M::new(0), M::new(0));
        assert_eq!(M::new(2) - M::new(1), M::new(1));

        assert_eq!(N(U128::from(0)) * N(U128::from(0)), N::new(U128::from(0)));
        assert_eq!(N(U128::from(2)) * N::from_integer(1), N::new(U128::from(2)));
        assert_eq!(
            R(U128::from(0)) * Q::new(U128::from(0)),
            R::new(U128::from(0))
        );
        assert_eq!(R(U128::from(2)) * Q::from_integer(1), R::new(U128::from(2)));
        assert_eq!(R(U128::from(0)) * M::new(0), R(U128::from(0)));
        assert_eq!(R(U128::from(2)) * M::from_integer(1), R(U128::from(2)));

        assert_eq!(N(U128::from(0)) / N(U128::from(1)), N::new(U128::from(0)));
        assert_eq!(N(U128::from(4)) / N::from_integer(2), N::new(U128::from(2)));
        assert_eq!(
            R(U128::from(0)) / Q::new(U128::from(1)),
            R::new(U128::from(0))
        );
        assert_eq!(R(U128::from(4)) / Q::from_integer(2), R::new(U128::from(2)));
        assert_eq!(R(U128::from(0)) / M::new(1), R::new(U128::from(0)));
        assert_eq!(R(U128::from(4)) / M::from_integer(2), R::new(U128::from(2)));
    }

    #[test]
    fn test_big_mul() {
        // precision
        {
            let a = Q::from_integer(1);
            let b = R::from_integer(1);
            let d = a.big_mul(b);
            let u = a.big_mul_up(b);
            assert_eq!(d, Q::from_integer(1));
            assert_eq!(u, Q::from_integer(1));

            let a = M::from_integer(1);
            let b = R::from_integer(1);
            let d = a.big_mul(b);
            let u = a.big_mul_up(b);
            assert_eq!(d, M::from_integer(1));
            assert_eq!(u, M::from_integer(1));
        }
        // simple
        {
            let a = Q::from_integer(2);
            let b = R::from_integer(3);
            let d = a.big_mul(b);
            let u = a.big_mul_up(b);
            assert_eq!(d, Q::from_integer(6));
            assert_eq!(u, Q::from_integer(6));

            let a = M::from_integer(2);
            let b = R::from_integer(3);
            let d = a.big_mul(b);
            let u = a.big_mul_up(b);
            assert_eq!(d, M::from_integer(6));
            assert_eq!(u, M::from_integer(6));
        }
        // big
        {
            let a = Q::new(U128::from(2u128.pow(127)));
            let b = N::from_integer(1);
            let d = a.big_mul(b);
            let u = a.big_mul_up(b);

            let expected = Q::new(U128::from(2u128.pow(127)));
            assert_eq!(d, expected);
            assert_eq!(u, expected);

            let a = M::new(2u128.pow(127));
            let b = N::from_integer(1);
            let d = a.big_mul(b);
            let u = a.big_mul_up(b);

            let expected = M::new(2u128.pow(127));
            assert_eq!(d, expected);
            assert_eq!(u, expected);
        }
        // random
        {
            let a = R::new(U128::from(879132));
            let b = Q::new(U128::from(9383));
            let d = a.big_mul(b);
            let u = a.big_mul_up(b);

            let expected = R(U128::from(824889555));
            assert_eq!(d, expected);
            assert_eq!(u, expected + R(U128::from(1)));

            let a = R::new(U128::from(879132));
            let b = M::new(9383);
            let d = a.big_mul(b);
            let u = a.big_mul_up(b);

            let expected = R(U128::from(824889555));
            assert_eq!(d, expected);
            assert_eq!(u, expected + R(U128::from(1)));
        }
    }

    #[test]
    fn test_big_div() {
        // precision
        {
            let a = Q::from_integer(1);
            let b = R::from_integer(1);
            let d = a.big_div(b);
            let u = a.big_div_up(b);
            assert_eq!(d, Q::from_integer(1));
            assert_eq!(u, Q::from_integer(1));

            let a = Q::from_integer(1);
            let b = M::from_integer(1);
            let d = a.big_div(b);
            let u = a.big_div_up(b);
            assert_eq!(d, Q::from_integer(1));
            assert_eq!(u, Q::from_integer(1));
        }
        // simple
        {
            let a = Q::from_integer(6);
            let b = R::from_integer(3);
            let d = a.big_div(b);
            let u = a.big_div_up(b);
            assert_eq!(d, Q::from_integer(2));
            assert_eq!(u, Q::from_integer(2));

            let a = Q::from_integer(6);
            let b = M::from_integer(3);
            let d = a.big_div(b);
            let u = a.big_div_up(b);
            assert_eq!(d, Q::from_integer(2));
            assert_eq!(u, Q::from_integer(2));
        }
        // big
        {
            let a = Q::new(U128::from(2u128.pow(127)));
            let b = R::from_integer(1u8);
            let d = a.big_div(b);
            let u = a.big_div_up(b);

            let expected = Q::new(U128::from(2u128.pow(127)));
            assert_eq!(d, expected);
            assert_eq!(u, expected);

            let a = Q::new(U128::from(2u128.pow(127)));
            let b = M::from_integer(1u8);
            let d = a.big_div(b);
            let u = a.big_div_up(b);

            let expected = Q::new(U128::from(2u128.pow(127)));
            assert_eq!(d, expected);
            assert_eq!(u, expected);
        }
        // random
        {
            let a = R::new(U128::from(824889555));
            let b = Q::new(U128::from(9383));
            let d = a.big_div(b);
            let u = a.big_div_up(b);

            let expected = R(U128::from(879131));
            assert_eq!(d, expected);
            assert_eq!(u, expected + R(U128::from(1)));

            let a = R::new(U128::from(824889555));
            let b = M::new(9383);
            let d = a.big_div(b);
            let u = a.big_div_up(b);

            let expected = R(U128::from(879131));
            assert_eq!(d, expected);
            assert_eq!(u, expected + R(U128::from(1)));
        }
    }

    #[test]
    fn tests_mul_to_number() {
        // basic
        {
            let a = Q::from_integer(1u8);
            let b = Q::from_integer(2u8);
            assert_eq!(a.big_mul_to_value(b), b.cast());
            assert_eq!(a.big_mul_to_value_up(b), b.cast());

            let a = M::from_integer(1u8);
            let b = M::from_integer(2u8);
            assert_eq!(a.big_mul_to_value(b), b.cast());
            assert_eq!(a.big_mul_to_value_up(b), b.cast());
        }
        // overflowing
        {
            let a = Q::new(U128::from(u128::MAX));
            let b = Q::new(U128::from(u128::MAX));
            // 1.15792089237316195423570985008687907853269984665640564039457584007913129639936 × 10^75
            // expected 11579208923731619542357098500868790785258941993179868711253083479304959321702
            assert_eq!(
                a.big_mul_to_value(b),
                U256::from_dec_str(
                    "11579208923731619542357098500868790785258941993179868711253083479304959321702"
                )
                .unwrap()
            );
            assert_eq!(
                a.big_mul_to_value_up(b),
                U256::from_dec_str(
                    "11579208923731619542357098500868790785258941993179868711253083479304959321703"
                )
                .unwrap()
            );

            let a = M::new(u128::MAX);
            let b = M::new(u128::MAX);
            // 1.15792089237316195423570985008687907853269984665640564039457584007913129639936 × 10^75
            // expected 11579208923731619542357098500868790785258941993179868711253083479304959321702
            assert_eq!(
                a.big_mul_to_value(b),
                U256::from_dec_str(
                    "11579208923731619542357098500868790785258941993179868711253083479304959321702"
                )
                .unwrap()
            );
            assert_eq!(
                a.big_mul_to_value_up(b),
                U256::from_dec_str(
                    "11579208923731619542357098500868790785258941993179868711253083479304959321703"
                )
                .unwrap()
            );
        }
    }

    #[test]
    fn test_big_div_by_number() {
        // basic
        {
            let a = Q::from_integer(4u8);
            let b = Q::from_integer(2u8);
            let big_type = b.cast();
            assert_eq!(a.big_div_by_number(big_type), b);
            assert_eq!(a.big_div_by_number_up(big_type), b);

            let a = M::from_integer(4u8);
            let b = M::from_integer(2u8);
            let big_type = b.cast();
            assert_eq!(a.big_div_by_number(big_type), b);
            assert_eq!(a.big_div_by_number_up(big_type), b);
        }
        // huge
        {
            let a = Q::new(U128::from(u128::MAX));
            let b = U256::from(u128::MAX) * U256::from(10) + U256::from(1);
            assert_eq!(a.big_div_by_number(b), Q::new(U128::from(0)));
            assert_eq!(a.big_div_by_number_up(b), Q::new(U128::from(1)));

            let a = M::new(u128::MAX);
            let b = U256::from(u128::MAX) * U256::from(10) + U256::from(1);
            assert_eq!(a.big_div_by_number(b), M::new(0));
            assert_eq!(a.big_div_by_number_up(b), M::new(1));
        }
        // random
        {
            let a = Q::new(U128::from(63424));
            let b = U256::from(157209);
            // real     0.403437462..
            // expected  4
            assert_eq!(a.big_div_by_number(b), Q::new(U128::from(4)));
            assert_eq!(a.big_div_by_number_up(b), Q::new(U128::from(5)));

            let a = M::new(63424);
            let b = U256::from(157209);
            // real     0.403437462..
            // expected  4
            assert_eq!(a.big_div_by_number(b), M::new(4));
            assert_eq!(a.big_div_by_number_up(b), M::new(5));
        }
    }

    #[test]
    fn test_mul_up() {
        // mul of little
        {
            let a = Q::new(U128::from(1));
            let b = Q::new(U128::from(1));
            assert_eq!(a.mul_up(b), Q::new(U128::from(1)));

            let a = M::new(1);
            let b = M::new(1);
            assert_eq!(a.mul_up(b), M::new(1));
        }
        // mul calculable without precision loss
        {
            let a = Q::from_integer(1);
            let b = Q::from_integer(3) / Q::new(U128::from(10));
            assert_eq!(a.mul_up(b), b);

            let a = M::from_integer(1);
            let b = M::from_integer(3) / M::new(10);
            assert_eq!(a.mul_up(b), b);
        }
        {
            let a = N(U128::from(1));
            let b = Q::from_integer(1);
            assert_eq!(a.mul_up(b), N(U128::from(1)));

            let a = M(1);
            let b = Q::from_integer(1);
            assert_eq!(a.mul_up(b), M(1));
        }
        {
            let a = N(U128::from(3));
            let b = Q::from_integer(3) / Q::from_integer(10);
            assert_eq!(a.mul_up(b), N(U128::from(1)));

            let a = M(3);
            let b = Q::from_integer(3) / Q::from_integer(10);
            assert_eq!(a.mul_up(b), M(1));
        }
    }

    #[test]
    fn test_div_up() {
        // div of zero
        {
            let a = Q::new(U128::from(0));
            let b = Q::new(U128::from(1));
            assert_eq!(a.div_up(b), Q::new(U128::from(0)));

            let a = M::new(0);
            let b = M::new(1);
            assert_eq!(a.div_up(b), M::new(0));
        }
        // div check rounding up
        {
            let a = Q::new(U128::from(1));
            let b = Q::from_integer(2);
            assert_eq!(a.div_up(b), Q::new(U128::from(1)));

            let a = M::new(1);
            let b = M::from_integer(2);
            assert_eq!(a.div_up(b), M::new(1));
        }
        // div big number
        {
            let a = R::new(U128::from(201));
            let b = R::from_integer(2);
            assert_eq!(a.div_up(b), R::new(U128::from(101)));

            let a = M::new(201);
            let b = M::from_integer(2);
            assert_eq!(a.div_up(b), M::new(101));
        }
        {
            let a = Q::new(U128::from(42));
            let b = R::from_integer(10);
            assert_eq!(a.div_up(b), Q::new(U128::from(5)));

            let a = M::new(42);
            let b = R::from_integer(10);
            assert_eq!(a.div_up(b), M::new(5));
        }
    }
}
