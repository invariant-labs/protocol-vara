use alloc::string::ToString;
use quote::quote;

use crate::utils::string_to_ident;
use crate::DecimalCharacteristics;

pub fn generate_others(characteristics: DecimalCharacteristics) -> proc_macro::TokenStream {
    let DecimalCharacteristics {
        struct_name,
        underlying_type,
        ..
    } = characteristics;

    let name_str = &struct_name.to_string();
    let underlying_str = &underlying_type.to_string();

    let module_name = string_to_ident("tests_others_", &name_str);

    proc_macro::TokenStream::from(quote!(
        impl<T: Decimal> Others<T> for #struct_name
        where
            T::U: TryInto<#underlying_type>,
        {
            fn mul_up(self, rhs: T) -> Self {
                let rhs_val: #underlying_type = rhs.get().try_into()
                    .unwrap_or_else(|_| core::panic!("decimal: rhs value cannot fit into `{}` type in {}::mul()", #underlying_str, #name_str));

                let one: #underlying_type = T::one().get().try_into()
                    .unwrap_or_else(|_| core::panic!("decimal: one value cannot fit into `{}` type in {}::div()", #underlying_str, #name_str));

                let almost_one: #underlying_type = T::almost_one().get().try_into()
                    .unwrap_or_else(|_| core::panic!("decimal: almost one value cannot fit into `{}` type in {}::div()", #underlying_str, #name_str));

                Self::new(
                    self.get().checked_mul(rhs_val)
                        .unwrap_or_else(|| core::panic!("decimal: mul overflow in method {}::mul_up()", #name_str))
                        .checked_add(almost_one)
                        .unwrap_or_else(|| core::panic!("decimal: add overflow in method {}::mul_up()", #name_str))
                        .checked_div(one)
                        .unwrap_or_else(|| core::panic!("decimal: div by 0 in method {}::mul_up()", #name_str))
                )
            }

            fn div_up(self, rhs: T) -> Self {
                let rhs_val: #underlying_type = rhs.get().try_into()
                    .unwrap_or_else(|_| core::panic!("decimal: rhs value cannot fit into `{}` type in {}::mul()", #underlying_str, #name_str));

                let one: #underlying_type = T::one().get().try_into()
                    .unwrap_or_else(|_| core::panic!("decimal: one value cannot fit into `{}` type in {}::div()", #underlying_str, #name_str));

                let almost_one: #underlying_type = T::almost_one().get().try_into()
                    .unwrap_or_else(|_| core::panic!("decimal: almost one value cannot fit into `{}` type in {}::div()", #underlying_str, #name_str));

                Self::new(
                    self.get().checked_mul(one)
                        .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::div_up()", #name_str))
                        .checked_add(rhs_val.checked_sub(#underlying_type::try_from(1u8).unwrap()).unwrap())
                        .unwrap_or_else(|| core::panic!("decimal: rhs value can't fit into `{}` type in {}::div_up()", #underlying_str, #name_str))
                        .checked_div(rhs_val)
                        .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::div_up()", #name_str))
                )
            }
        }

        impl OthersSameType for #struct_name {
            fn sub_abs(self, rhs: Self) -> Self {
                if self.get() > rhs.get() {
                    self - rhs
                } else {
                    rhs - self
                }
            }
        }

        impl core::fmt::Display for #struct_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                if Self::scale() > 0 {
                    let mut decimal_places: #underlying_type = self.get().checked_rem(Self::one().get()).unwrap();
                    let mut non_zero_tail = 0;

                    while decimal_places > #underlying_type::try_from(0u8).unwrap() {
                        non_zero_tail += 1;
                        decimal_places /= #underlying_type::try_from(10u8).unwrap();
                    }

                    write!(
                        f,
                        "{}.{}{}",
                        self.get().checked_div(Self::one().get()).unwrap(),
                        "0".repeat((Self::scale() - non_zero_tail).into()),
                        self.get().checked_rem(Self::one().get()).unwrap()
                    )
                } else {
                    write!(f, "{}", self.get())
                }
            }
        }


        #[cfg(test)]
        pub mod #module_name {
            use super::*;

            #[test]
            fn test_mul_up() {
                let a = #struct_name::new(#underlying_type::from(1u8));
                let b = #struct_name::one();
                assert_eq!(a.mul_up(b), a);
            }

            #[test]
            fn test_div_up() {
                let a = #struct_name::new(#underlying_type::from(1u8));
                let b = #struct_name::one();
                assert_eq!(a.div_up(b), a);
            }

            #[test]
            fn test_sub_abs() {
                let a = #struct_name::new(#underlying_type::from(1u8));
                let b = #struct_name::new(#underlying_type::from(2u8));
                assert_eq!(a.sub_abs(b), a);
                assert_eq!(b.sub_abs(a), a);
            }
        }
    ))
}