use alloc::string::ToString;
use quote::quote;

use crate::utils::string_to_ident;
use crate::DecimalCharacteristics;

pub fn generate_by_number(characteristics: DecimalCharacteristics) -> proc_macro::TokenStream {
    let DecimalCharacteristics {
        struct_name,
        big_type,
        underlying_type,
        ..
    } = characteristics;

    let name_str = &struct_name.to_string();
    let big_str = &big_type.to_string();

    let module_name = string_to_ident("tests_by_number_", &name_str);

    proc_macro::TokenStream::from(quote!(
        impl ByNumber<#big_type> for #struct_name {
            fn big_div_by_number(self, rhs: #big_type) -> Self {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_self_one: #big_type = Self::one().cast::<#big_type>();

                Self::new(#struct_name::from_value((big_self
                    .checked_mul(big_self_one)
                    .unwrap()
                    .checked_div(rhs)
                    .unwrap()
                )))

            }

            fn checked_big_div_by_number(self, rhs: #big_type) -> core::result::Result<Self, alloc::string::String> {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_self_one: #big_type = Self::one().cast::<#big_type>();

                Ok(Self::new(#struct_name::checked_from_value(big_self
                    .checked_mul(big_self_one)
                    .ok_or_else(|| alloc::format!("decimal: lhs value can't fit into `{}` type in {}::checked_big_div_by_number()", #big_str, #name_str))?
                    .checked_div(rhs)
                    .ok_or_else(|| alloc::format!("decimal: lhs value can't fit into `{}` type in {}::checked_big_div_by_number()", #big_str, #name_str))?
                    )?
                ))
            }

            fn big_div_by_number_up(self, rhs: #big_type) -> Self {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_self_one: #big_type = Self::one().cast::<#big_type>();

                Self::new(#struct_name::from_value(big_self
                    .checked_mul(big_self_one)
                    .unwrap()
                    .checked_add(rhs.checked_sub(#big_type::from(1u8)).unwrap())
                    .unwrap()
                    .checked_div(rhs)
                    .unwrap()
                ))
            }

            fn checked_big_div_by_number_up(self, rhs: #big_type) -> core::result::Result<Self, alloc::string::String> {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_self_one: #big_type = Self::one().cast::<#big_type>();

                Ok(Self::new(#struct_name::checked_from_value(big_self
                    .checked_mul(big_self_one)
                    .ok_or_else(|| alloc::format!("decimal: lhs value can't fit into `{}` type in {}::checked_big_div_by_number_up()", #big_str, #name_str))?
                    .checked_add(rhs.checked_sub(#big_type::from(1u8)).unwrap())
                    .ok_or_else(|| alloc::format!("decimal: lhs value can't fit into `{}` type in {}::checked_big_div_by_number_up()", #big_str, #name_str))?
                    .checked_div(rhs)
                    .ok_or_else(|| alloc::format!("decimal: lhs value can't fit into `{}` type in {}::checked_big_div_by_number_up()", #big_str, #name_str))?
                )?))
            }
        }

        impl<T: Decimal> ToValue<T, #big_type> for #struct_name
        where
            #big_type: UintCast<<T as Decimal>::U>,
            T: Decimal + Conversion
        {
            fn big_mul_to_value(self, rhs: T) -> #big_type {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_rhs: #big_type = rhs.cast::<#big_type>();
                let big_rhs_one: #big_type = T::one().cast::<#big_type>();

                big_self
                    .checked_mul(big_rhs)
                    .unwrap()
                    .checked_div(big_rhs_one)
                    .unwrap()
            }

            fn big_mul_to_value_up(self, rhs: T) -> #big_type {
                let big_self: #big_type = self.cast::<#big_type>();
                let big_rhs: #big_type = rhs.cast::<#big_type>();
                let big_rhs_one: #big_type = T::one().cast::<#big_type>();
                let big_rhs_almost_one: #big_type = T::almost_one().cast::<#big_type>();

                big_self
                    .checked_mul(big_rhs)
                    .unwrap()
                    .checked_add(big_rhs_almost_one)
                    .unwrap()
                    .checked_div(big_rhs_one)
                    .unwrap()

                }
        }

        #[cfg(test)]
        pub mod #module_name {
            use super::*;

            #[test]
            fn test_big_div_up_by_number () {
                let a = #struct_name::new(#underlying_type::from(2u8));
                let b: #big_type = #struct_name::one().cast();
                assert_eq!(a.big_div_by_number(b), #struct_name::new(#underlying_type::from(2u8)));
                assert_eq!(a.big_div_by_number_up(b), #struct_name::new(#underlying_type::from(2u8)));
            }

            #[test]
            fn test_checked_big_div_by_number() {
                let a = #struct_name::new(#underlying_type::from(2u8));
                let b: #big_type = #struct_name::one().cast();
                assert_eq!(a.checked_big_div_by_number(b), Ok(#struct_name::new(#underlying_type::from(2u8))));
            }

            #[test]
            fn test_checked_big_div_by_number_up() {
                let a = #struct_name::new(#underlying_type::from(2u8));
                let b: #big_type = #struct_name::one().cast();
                assert_eq!(a.checked_big_div_by_number_up(b), Ok(#struct_name::new(#underlying_type::from(2u8))));
            }

            #[test]
            fn test_big_mul_to_value () {
                let a = #struct_name::new(#underlying_type::from(2u8));
                let b = #struct_name::one();
                let c: #big_type = #big_type::from(2u8);
                assert_eq!(a.big_mul_to_value(b), c);
                assert_eq!(a.big_mul_to_value_up(b), c);
            }
        }
    ))
}