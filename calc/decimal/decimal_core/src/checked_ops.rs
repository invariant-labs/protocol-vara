use alloc::string::ToString;
use quote::quote;

use crate::utils::string_to_ident;
use crate::DecimalCharacteristics;

pub fn generate_checked_ops(characteristics: DecimalCharacteristics) -> proc_macro::TokenStream {
    let DecimalCharacteristics {
        struct_name,
        underlying_type,
        ..
    } = characteristics;

    let name_str = &struct_name.to_string();
    let module_name = string_to_ident("tests_checked_ops_", &name_str);

    proc_macro::TokenStream::from(quote!(
        impl CheckedOps for #struct_name {
            fn checked_add(self, rhs: Self) -> core::result::Result<Self, alloc::string::String> {
                Ok(Self::new(
                    self.get().checked_add(rhs.get())
                    .ok_or_else(|| "checked_add: (self + rhs) additional overflow")?
                ))
            }

            fn checked_sub(self, rhs: Self) -> core::result::Result<Self, alloc::string::String> {
                Ok(Self::new(
                    self.get().checked_sub(rhs.get())
                    .ok_or_else(|| "checked_sub: (self - rhs) subtraction underflow")?
                ))
            }

            fn checked_div(self, rhs: Self) -> core::result::Result<Self, alloc::string::String> {
                Ok(Self::new(
                        self.get()
                        .checked_mul(Self::one().get())
                        .ok_or_else(|| "checked_div: (self * Self::one()) multiplication overflow")?
                        .checked_div(rhs.get())
                        .ok_or_else(|| "checked_div: ((self * Self::one()) / rhs) division by zero")?
                    )
                )
            }
        }

        #[cfg(test)]
        pub mod #module_name {
            use super::*;

            #[test]
            fn test_checked_add() {
                let a = #struct_name::new(#underlying_type::try_from(24u8).unwrap());
                let b = #struct_name::new(#underlying_type::try_from(11u8).unwrap());

                assert_eq!(a.checked_add(b), Ok(#struct_name::new(#underlying_type::try_from(35u8).unwrap())));
            }

            #[test]
            fn test_overflow_checked_add() {
                let max = #struct_name::max_instance();
                let result = max.checked_add(#struct_name::new(#underlying_type::try_from(1u8).unwrap()));

                assert_eq!(result, Err(alloc::string::String::from("checked_add: (self + rhs) additional overflow")));
            }

            #[test]
            fn test_checked_sub() {
                let a = #struct_name::new(#underlying_type::try_from(35u8).unwrap());
                let b = #struct_name::new(#underlying_type::try_from(11u8).unwrap());

                assert_eq!(a.checked_sub(b), Ok(#struct_name::new(#underlying_type::try_from(24u8).unwrap())));
            }

            #[test]
            fn test_checked_div() {
                let a = #struct_name::new(#underlying_type::try_from(2u8).unwrap());
                let b = #struct_name::new(#underlying_type::try_from(#struct_name::one().get()).unwrap());
                assert_eq!(a.checked_div(b), Ok(#struct_name::new(#underlying_type::try_from(2u8).unwrap())));
            }

            #[test]
            fn test_0_checked_div() {
                let a = #struct_name::new(#underlying_type::try_from(47u8).unwrap());
                let b = #struct_name::new(#underlying_type::try_from(0u8).unwrap());
                let result = a.checked_div(b);
                assert!(result.is_err());
            }

            #[test]
            fn test_underflow_checked_sub() {
                let min = #struct_name::new(#underlying_type::try_from(0u8).unwrap());
                let result = min.checked_sub(#struct_name::new(#underlying_type::try_from(1u8).unwrap()));

                assert_eq!(result, Err(alloc::string::String::from("checked_sub: (self - rhs) subtraction underflow")));
            }
        }
    ))
}