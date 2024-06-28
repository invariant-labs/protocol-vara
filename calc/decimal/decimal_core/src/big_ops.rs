use alloc::string::ToString;
use quote::quote;

use crate::utils::string_to_ident;
use crate::DecimalCharacteristics;
pub fn generate_big_ops(characteristics: DecimalCharacteristics) -> proc_macro::TokenStream {
    let DecimalCharacteristics {
        struct_name,
        big_type,
        underlying_type,
        ..
    } = characteristics;

    let name_str = &struct_name.to_string();
    let big_str = &big_type.to_string();

    let module_name = string_to_ident("tests_big_ops_", &name_str);
    proc_macro::TokenStream::from(quote!(
        impl<T: Decimal> BigOps<T> for #struct_name
        where
        T: Decimal + alloc::fmt::Debug + Conversion,
        T::U: AsRef<[u64]>,
        {
            fn big_mul(self, rhs: T) -> Self {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_rhs: #big_type = rhs.cast::<#big_type>();
                let big_one: #big_type = T::one().cast::<#big_type>();

                Self::new(#struct_name::from_value(big_self
                    .checked_mul(big_rhs)
                    .unwrap_or_else(|| core::panic!("decimal: lhs value can't fit into `{}` type in {}::big_mul()", #big_str, #name_str))
                    .checked_div(big_one)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_mul()", #name_str))
                ))
            }

            fn big_mul_up(self, rhs: T) -> Self {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_rhs: #big_type = rhs.cast::<#big_type>();
                let big_one: #big_type = T::one().cast::<#big_type>();
                let big_almost_one: #big_type = T::almost_one().cast::<#big_type>();

                Self::new(#struct_name::from_value(big_self
                    .checked_mul(big_rhs)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_mul_up()", #name_str))
                    .checked_add(big_almost_one)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_mul_up()", #name_str))
                    .checked_div(big_one)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_mul_up()", #name_str))
                ))
            }

            fn big_div(self, rhs: T) -> Self {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_rhs: #big_type = rhs.cast::<#big_type>();
                let big_one: #big_type = T::one().cast::<#big_type>();

                Self::new(#struct_name::from_value(big_self
                    .checked_mul(big_one)
                    .unwrap_or_else(|| core::panic!("decimal: lhs value can't fit into `{}` type in {}::big_div()", #big_str, #name_str))
                    .checked_div(big_rhs)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_div()", #name_str))
                ))
            }

            fn checked_big_div(self, rhs: T) -> core::result::Result<Self, alloc::string::String> {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_rhs: #big_type = rhs.cast::<#big_type>();
                let big_one: #big_type = T::one().cast::<#big_type>();

                Ok(Self::new(
                    #struct_name::checked_from_value(
                        big_self
                            .checked_mul(big_one)
                            .ok_or_else(|| alloc::format!("decimal: lhs value can't fit into `{}` type in {}::checked_big_div()", #big_str, #name_str))?
                            .checked_div(big_rhs)
                            .ok_or_else(|| alloc::format!("decimal: lhs value can't fit into `{}` type in {}::checked_big_div()", #big_str, #name_str))?
                )?))
            }

            fn big_div_up(self, rhs: T) -> Self {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_rhs: #big_type = rhs.cast::<#big_type>();
                let big_one: #big_type = T::one().cast::<#big_type>();

                Self::new(#struct_name::from_value(big_self
                    .checked_mul(big_one)
                    .unwrap_or_else(|| core::panic!("decimal: lhs value can't fit into `{}` type in {}::big_div_up()", #big_str, #name_str))
                    .checked_add(big_rhs)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_div_up()", #name_str))
                    .checked_sub(#big_type::from(1))
                    .unwrap_or_else(|| core::panic!("decimal: underflow in method {}::big_div_up()", #name_str))
                    .checked_div(big_rhs)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_div_up()", #name_str))
                ))
            }
        }

        #[cfg(test)]
        pub mod #module_name {
            use super::*;

            #[test]
            fn test_big_mul () {
                let a = #struct_name::new(#underlying_type::from(2));
                let b = #struct_name::one();
                assert_eq!(a.big_mul(b), #struct_name::new(#underlying_type::from(2)));
            }

            #[test]
            fn test_big_mul_up () {
                let a = #struct_name::new(#underlying_type::from(2));
                let b = #struct_name::one();
                assert_eq!(a.big_mul_up(b), a);
            }

            #[test]
            fn test_big_div () {
                let a = #struct_name::new(#underlying_type::from(2));
                let b = #struct_name::one();
                assert_eq!(a.big_div(b), #struct_name::new(#underlying_type::from(2)));
            }

            #[test]
            fn test_checked_big_div () {
                let a = #struct_name::new(#underlying_type::from(29));
                let b = #struct_name::one();
                assert_eq!(a.big_div(b), a);
            }

            #[test]
            fn test_big_div_up () {
                let a = #struct_name::new(#underlying_type::from(2));
                let b = #struct_name::one();
                assert_eq!(a.big_div_up(b), #struct_name::new(#underlying_type::from(2)));
            }
        }
    ))
}