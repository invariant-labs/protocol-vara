use alloc::string::ToString;
use quote::quote;

use crate::utils::string_to_ident;
use crate::DecimalCharacteristics;

pub fn generate_base(characteristics: DecimalCharacteristics) -> proc_macro::TokenStream {
    let DecimalCharacteristics {
        struct_name,
        underlying_type,
        big_type,
        scale: parsed_scale,
        field_name,
        ..
    } = characteristics;

    let name_str = &struct_name.to_string();
    let module_name = string_to_ident("tests_base_", &name_str);

    proc_macro::TokenStream::from(quote!(
        impl Decimal for #struct_name {
            type U = #underlying_type;

            fn get(&self) -> #underlying_type {
                self.#field_name
            }

            fn new(value: Self::U) -> Self {
                let mut created = #struct_name::default();
                created.#field_name = value;
                created
            }

            fn max_value() -> Self::U {
                Self::U::MAX
            }

            fn max_instance() -> Self {
                Self::new(Self::max_value())
            }

            fn here<T: TryFrom<Self::U>>(&self) -> T {
                match T::try_from(self.#field_name) {
                    Ok(v) => v,
                    Err(_) => core::panic!("could not parse {} to {}", "T", "u8"),
                }
            }

            fn scale() -> u8 {
                #parsed_scale
            }

            fn checked_one() -> Result<Self, alloc::string::String> {
                let base = #underlying_type::try_from(10u8).map_err(|_| "checked_one: cannot create underlying_type from u8")?;
                Ok(Self::new(
                    base.checked_pow(
                        Self::scale().try_into().map_err(|_| "checked_one: cannot convert scale() to decimal exponent")?
                    ).ok_or_else(|| "checked_one: cannot calculate 10.pow(scale())")?
                ))
            }

            fn one() -> Self {
                Self::checked_one().unwrap()
            }

            fn checked_almost_one() -> Result<Self, alloc::string::String> {
                let min_diff = #underlying_type::try_from(1u8).map_err(|_| "checked_almost_one: cannot create underlying_type from u8")?;
                let one = Self::checked_one()?;

                Ok(Self::new(
                    one.get().checked_sub(min_diff).ok_or_else(|| "checked_almost_one: cannot calculate (ONE - 1)")?
                ))
            }

            fn almost_one() -> Self {
                Self::checked_almost_one().unwrap()
            }
        }

        impl Conversion for #struct_name
        where
        {
            fn cast<T: Default
                    + AsRef<[u64]>
                    + From<u64>
                    + core::ops::Shl<usize, Output = T>
                    + core::ops::BitOrAssign,
                >(self) -> T {
                    Self::checked_cast(self).unwrap()
            }

            fn checked_cast<T: Default
                    + AsRef<[u64]>
                    + From<u64>
                    + core::ops::Shl<usize, Output = T>
                    + core::ops::BitOrAssign,
                >(self) -> Result<T, alloc::string::String> {
                    let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into().unwrap();
                    let mut result = T::default();
                    let result_length: usize = result.as_ref().len();

                    if self_bytes.len() > result_length {
                        let (self_bytes, remaining_bytes) = self_bytes.split_at_mut(result_length);
                        if remaining_bytes.iter().any(|&x| x != 0) {
                            return Err(alloc::string::String::from("Overflow while casting."))
                        }
                    }

                    for (index, &value) in self_bytes.iter().enumerate() {
                        result |= (T::from(value) << (index * 64));
                    }
                    Ok(result)
            }

            fn from_value<T, R>(from: R) -> T
            where
                T: Default
                    + AsRef<[u64]>
                    + From<u64>
                    + core::ops::Shl<usize, Output = T>
                    + core::ops::BitOrAssign,
                R: Default
                    + AsRef<[u64]>
                    + From<u64>
                    + core::ops::Shl<usize, Output = R>
                    + core::ops::BitOrAssign,
            {
                Self::checked_from_value(from).unwrap()
            }

            fn checked_from_value<T,R>(from:R) -> Result<T, alloc::string::String>
            where
            T: Default
                + AsRef<[u64]>
                + From<u64>
                + core::ops::Shl<usize, Output = T>
                + core::ops::BitOrAssign,
            R: Default
                + AsRef<[u64]>
                + From<u64>
                + core::ops::Shl<usize, Output = R>
                + core::ops::BitOrAssign,
            {
                let mut self_bytes: alloc::vec::Vec<u64> = from.as_ref().try_into().unwrap();
                let mut result = T::default();
                let result_length: usize = result.as_ref().len();
                if self_bytes.len() > result_length {
                    let (self_bytes, remaining_bytes) = self_bytes.split_at_mut(result_length);
                    if remaining_bytes.iter().any(|&x| x != 0) {
                        return Err(alloc::string::String::from("Overflow while casting from value."))
                    }
                }
                for (index, &value) in self_bytes.iter().enumerate() {
                    result |= (T::from(value) << (index * 64));
                }
                Ok(result)
            }
        }

        #[cfg(test)]
        pub mod #module_name {
            use super::*;

            #[test]
            fn test_new() {
                let decimal = #struct_name::new(#underlying_type::default());
                assert_eq!(#underlying_type::default(), decimal.get());
            }

            #[test]
            fn test_max_instance() {
                let decimal = #struct_name::max_instance();
                assert_eq!(#underlying_type::MAX, decimal.get());
            }

            #[test]
            fn test_one() {
                let decimal = #struct_name::one();
                assert_eq!(
                    decimal.get(),
                    #underlying_type::try_from(10u8).unwrap().checked_pow(#parsed_scale.into()).unwrap());
            }

            #[test]
            fn test_almost_one() {
                let decimal = #struct_name::almost_one();
                let min_diff_val = #underlying_type::try_from(1u8).unwrap();
                assert_eq!(
                    decimal.get(),
                    #underlying_type::try_from(10u8).unwrap().checked_pow(#parsed_scale.into()).unwrap().checked_sub(min_diff_val).unwrap()
                );
            }

            #[test]
            fn test_cast() {
                let one_underlying = #underlying_type::from(1u8);
                let one_big_type = #big_type::from(1u8);

                let one_decimal = #struct_name::new(one_underlying);
                let underlying_from_decimal: #underlying_type = one_decimal.cast::<#underlying_type>();
                assert_eq!(
                    one_underlying,
                    underlying_from_decimal
                );

                let big_type_from_decimal: #big_type = one_decimal.cast::<#big_type>();
                assert_eq!(
                    one_big_type,
                    big_type_from_decimal
                );
            }

            #[test]
            fn test_from_value() {
                let one_underlying = #underlying_type::from(1u8);
                let one_big_type = #big_type::from(1u8);

                let converted_big_one: #big_type = #struct_name::from_value::<#big_type, #underlying_type>(one_underlying);
                assert_eq!(one_big_type, converted_big_one);

                let converted_underlying_one: #underlying_type = #struct_name::from_value::<#underlying_type,#big_type>(converted_big_one);
                assert_eq!(one_underlying, converted_underlying_one);
            }
        }
    ))
}
