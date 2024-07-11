use alloc::string::ToString;
use quote::quote;

use crate::utils::string_to_ident;
use crate::DecimalCharacteristics;

pub fn generate_ops(characteristics: DecimalCharacteristics) -> proc_macro::TokenStream {
    let DecimalCharacteristics {
        struct_name,
        underlying_type,
        ..
    } = characteristics;

    let name_str = &struct_name.to_string();
    let underlying_str = &underlying_type.to_string();

    let module_name = string_to_ident("tests_", &name_str);

    proc_macro::TokenStream::from(quote!(
        impl core::ops::Add for #struct_name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self::new(self.get()
                    .checked_add(rhs.get())
                    .unwrap_or_else(|| panic!("decimal: overflow in method {}::add()", #name_str))
                )
            }
        }

        impl core::ops::Sub for #struct_name {
            type Output = #struct_name;

            fn sub(self, rhs: Self) -> #struct_name {
                Self::new(self.get()
                    .checked_sub(rhs.get())
                    .unwrap_or_else(|| panic!("decimal: overflow in method {}::sub()", #name_str))
                )
            }
        }

        impl<T: Decimal> core::ops::Mul<T> for #struct_name
        where
            T::U: TryInto<#underlying_type>,
        {
            type Output = #struct_name;

            fn mul(self, rhs: T) -> Self {
                let rhs_val: #underlying_type = rhs.get().try_into()
                    .unwrap_or_else(|_| core::panic!("decimal: rhs value cannot fit into `{}` type in {}::mul()", #underlying_str, #name_str));

                let one: #underlying_type = T::one().get().try_into()
                    .unwrap_or_else(|_| core::panic!("decimal: one value cannot fit into `{}` type in {}::mul()", #underlying_str, #name_str));

                let both_decimals: #underlying_type = self.get().checked_mul(rhs_val)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::mul()", #name_str));

                let result: #underlying_type = both_decimals.checked_div(one)
                    .unwrap_or_else(|| core::panic!("decimal: div by 0 in {}::mul()", #name_str));

                Self::new(result)
            }
        }

        impl<T: Decimal> core::ops::Div<T> for #struct_name
        where
            T::U: TryInto<#underlying_type>,
        {
            type Output = Self;

            fn div(self, rhs: T) -> Self {
                let rhs_val: #underlying_type = rhs.get().try_into()
                    .unwrap_or_else(|_| core::panic!("decimal: rhs value cannot fit into `{}` type in {}::div()", #underlying_str, #name_str));

                let one: #underlying_type = T::one().get().try_into()
                    .unwrap_or_else(|_| core::panic!("decimal: one value cannot fit into `{}` type in {}::div()", #underlying_str, #name_str));

                let extended_self: #underlying_type = self.get().checked_mul(one)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::div()", #name_str));

                let result: #underlying_type = extended_self.checked_div(rhs_val)
                    .unwrap_or_else(|| core::panic!("decimal: div by 0 in {}::div()", #name_str));

                Self::new(result)
            }
        }

        impl core::ops::AddAssign for #struct_name {
            fn add_assign(&mut self, rhs: Self)  {
                *self = *self + rhs
            }
        }

        impl core::ops::SubAssign for #struct_name {
            fn sub_assign(&mut self, rhs: Self)  {
                *self = *self - rhs
            }
        }

        impl core::ops::MulAssign for #struct_name {
            fn mul_assign(&mut self, rhs: Self)  {
                *self = *self * rhs
            }
        }

        impl core::ops::DivAssign for #struct_name {
            fn div_assign(&mut self, rhs: Self)  {
                *self = *self / rhs
            }
        }


        #[cfg(test)]
        pub mod #module_name {
            use super::*;

            #[test]
            fn test_add () {
                let one_unit = #underlying_type::from(1u8);
                let two_unit = #underlying_type::from(2u8);

                let mut a = #struct_name::new(one_unit);
                let b = #struct_name::new(one_unit);
                assert_eq!(a + b, #struct_name::new(two_unit));
                a += b;
                assert_eq!(a, #struct_name::new(two_unit));
            }

            #[test]
            fn test_sub () {
                let zero = #underlying_type::from(0u8);
                let one_unit = #underlying_type::from(1u8);

                let mut a = #struct_name::new(one_unit);
                let b = #struct_name::new(one_unit);
                assert_eq!(a - b, #struct_name::new(zero));
                a -= b;
                assert_eq!(a, #struct_name::new(zero));
            }

            #[test]
            fn test_mul () {
                let two_unit = #underlying_type::from(2u8);

                let mut a = #struct_name::new(two_unit);
                let b = #struct_name::one();
                assert_eq!(a * b, #struct_name::new(two_unit));
                a *= b;
                assert_eq!(a, #struct_name::new(two_unit));
            }

            #[test]
            fn test_div () {
                let two_unit = #underlying_type::from(2u8);

                let mut a = #struct_name::new(two_unit);
                let b = #struct_name::one();
                assert_eq!(a / b, #struct_name::new(two_unit));
                a /= b;
                assert_eq!(a, #struct_name::new(two_unit));
            }
        }
    ))
}