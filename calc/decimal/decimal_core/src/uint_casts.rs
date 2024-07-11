use crate::utils::string_to_ident;
use proc_macro2;
use quote::{quote, TokenStreamExt};
pub struct Uint(pub syn::Ident, pub syn::LitInt);

impl syn::parse::Parse for Uint {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        Ok(Uint(input.parse()?, input.parse()?))
    }
}

pub struct UintsCastsInput {
    pub uints: syn::punctuated::Punctuated<Uint, syn::Token![,]>,
}

impl syn::parse::Parse for UintsCastsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        Ok(UintsCastsInput {
            uints: syn::punctuated::Punctuated::<Uint, syn::Token![,]>::parse_separated_nonempty(
                input,
            )?,
        })
    }
}

fn overflow_chunks(
    outer_range: impl Iterator<Item = usize> + Clone,
    param_name: proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let max_chunk = match outer_range.clone().max() {
      Some(size) => size,
      None => return Default::default()
    };

    outer_range.fold(proc_macro2::TokenStream::new(), |mut acc, i| {
        if i != max_chunk {
            acc.append_all(quote! { #param_name.0[#i] | });
        } else {
            acc.append_all(quote! { #param_name.0[#i] });
        }
        acc
    })
}
pub fn validate_uint(
    new_uint: proc_macro2::Ident,
    new_chunks_count: usize,
) -> proc_macro2::TokenStream {
    // compile time check of the provided length
    let test_ident = proc_macro2::Ident::new(
        &alloc::format!("test_size_{}", new_uint),
        proc_macro2::Span::call_site(),
    );
    let test_chunks = (0..new_chunks_count).fold(proc_macro2::TokenStream::new(), |mut acc, _| {
        acc.append_all(quote! { 0, });
        acc
    });
    quote! {
        mod #test_ident {
            use super::#new_uint;
            fn test_size() {
                #new_uint([#test_chunks]);
            }
        }
    }
}
pub fn impl_uint_casts(
    uints: alloc::vec::Vec<(proc_macro2::Ident, usize)>,
    new_uint: proc_macro2::Ident,
    new_chunks_count: usize,
) -> proc_macro2::TokenStream {
    if new_chunks_count < 2 {
        // U64 uint is just raw u64 wrapper
        panic!("Lowest supported chunks count is 2")
    }

    let mut expanded = proc_macro2::TokenStream::new();

    let param_name = string_to_ident("", "val");
    let cast_function_name = string_to_ident("", "uint_cast");
    let cast_trait_name = string_to_ident("", "UintCast");
    let checked_cast_trait_name = string_to_ident("", "UintCheckedCast");
    let checked_cast_function_name = string_to_ident("", "uint_checked_cast");

    for (uint, chunks_count) in uints.iter() {
        let chunks_count = *chunks_count;

        let (max_chunk, max_common_chunk, bigger_type, smaller_type) =
            if new_chunks_count == chunks_count {
              panic!("uints {}, {} are identical, aliases are not supported", uint, new_uint)
            }
            else if new_chunks_count > chunks_count {
                (
                    new_chunks_count,
                    chunks_count,
                    new_uint.clone(),
                    uint.clone(),
                )
            } else {
                (
                    chunks_count,
                    new_chunks_count,
                    uint.clone(),
                    new_uint.clone(),
                )
            };

        let inner_range = 0..max_common_chunk;
        let outer_range = max_common_chunk..max_chunk;

        let common_chunks = inner_range.fold(proc_macro2::TokenStream::new(), |mut acc, i| {
            acc.append_all(quote! { #param_name.0[#i], });
            acc
        });

        let overflow_chunks = overflow_chunks(outer_range.clone(), param_name.clone());

        if new_chunks_count <= chunks_count {
            let empty_chunks =
                outer_range
                    .clone()
                    .fold(proc_macro2::TokenStream::new(), |mut acc, _| {
                        acc.append_all(quote! { 0, });
                        acc
                    });

            let cast_up = quote! {
                impl #cast_trait_name<#smaller_type> for #bigger_type {
                    fn #cast_function_name(#param_name: #smaller_type)-> #bigger_type {
                        #bigger_type([#common_chunks #empty_chunks])
                    }
                }
                impl #checked_cast_trait_name <#smaller_type> for #bigger_type {
                    fn #checked_cast_function_name(#param_name: #smaller_type)-> Result<#bigger_type, String> {
                        Ok(#bigger_type::#cast_function_name(#param_name))
                    }
                }
            };

            let error_message = alloc::format!("Failed to cast {} to {}", smaller_type, bigger_type);
            let cast_down: proc_macro2::TokenStream = quote! {
                impl #checked_cast_trait_name <#bigger_type> for #smaller_type {
                    fn #checked_cast_function_name(#param_name: #bigger_type)-> Result<#smaller_type, String> {
                        if #overflow_chunks != 0 {
                            return Err(#error_message.to_string());
                        }

                        Ok(#smaller_type([#common_chunks]))
                    }
                }
                impl #cast_trait_name<#bigger_type> for #smaller_type {
                    fn #cast_function_name(#param_name: #bigger_type)-> #smaller_type {
                        #smaller_type::#checked_cast_function_name(#param_name).unwrap()
                    }
                }
            };
            expanded.append_all(cast_up);
            expanded.append_all(cast_down);
        }
    }
    expanded
}

pub fn impl_primitive_casts(
    uint: proc_macro2::Ident,
    chunk_count: usize,
) -> proc_macro2::TokenStream {
    let mut expanded: proc_macro2::TokenStream = proc_macro2::TokenStream::new();

    let param_name = string_to_ident("", "val");
    let cast_function_name = string_to_ident("", "uint_cast");
    let cast_trait_name = string_to_ident("", "UintCast");
    let checked_cast_trait_name = string_to_ident("", "UintCheckedCast");
    let checked_cast_function_name = string_to_ident("", "uint_checked_cast");

    // upcasts
    // upcast from primitive up to u64
    // i32 cast is added to allow not specifying the type in from_integer function in decimal factories
    // uints from the uint crate panic in cases where i32 is negative so no check is required
    ["u8", "u16", "i32", "u32", "u64"]
        .iter()
        .map(|v| proc_macro2::Ident::new(v, proc_macro2::Span::call_site()))
        .for_each(|primitive| {
            expanded.append_all(quote! {
              impl #cast_trait_name<#primitive> for #uint {
                fn #cast_function_name(#param_name: #primitive)-> #uint {
                    #uint::from(#param_name)
                }
              }
              impl #checked_cast_trait_name <#primitive> for #uint {
                  fn #checked_cast_function_name(#param_name: #primitive)-> Result<#uint, String> {
                      Ok(#uint::#cast_function_name(#param_name))
                  }
              }
            });
        });
    // upcast from u128
    if chunk_count >= 2 {
        expanded.append_all(quote! {
          impl #cast_trait_name<u128> for #uint {
            fn #cast_function_name(#param_name: u128)-> #uint {
              #uint::from(#param_name)
            }
          }
          impl #checked_cast_trait_name <u128> for #uint {
            fn #checked_cast_function_name(#param_name: u128)-> Result<#uint, String> {
              return Ok(#uint::from(#param_name))
            }
          }
        });
    }

    // exact casts
    match chunk_count {
        // exact cast to u64
        1 => {
            expanded.append_all(quote! {
              impl #checked_cast_trait_name <#uint> for u64 {
                fn #checked_cast_function_name(#param_name: #uint)-> Result<u64, String> {
                  return Ok(#param_name.low_u64())
                }
              }
              impl #cast_trait_name<#uint> for u64 {
                fn #cast_function_name(#param_name: #uint)-> u64 {
                  #param_name.low_u64()
                }
              }
            });
        }
        // exact cast to u128
        2 => {
            expanded.append_all(quote! {
              impl #checked_cast_trait_name <#uint> for u128 {
                fn #checked_cast_function_name(#param_name: #uint)-> Result<u128, String> {
                  return Ok(#param_name.low_u128())
                }
              }
              impl #cast_trait_name<#uint> for u128 {
                fn #cast_function_name(#param_name: #uint)-> u128 {
                  #param_name.low_u128()
                }
              }
            });
        }
        _ => {}
    }
    // downcasts
    // downcast to u32 or below
    ["u8", "u16", "u32"]
        .iter()
        .map(|v| proc_macro2::Ident::new(v, proc_macro2::Span::call_site()))
        .for_each(|primitive| {
            let down_cast_error = alloc::format!("Failed to cast {} to {}", uint, primitive);
            expanded.append_all(quote! {
              impl #checked_cast_trait_name <#uint> for #primitive {
                fn #checked_cast_function_name(#param_name: #uint)-> Result<#primitive, String> {
                  if #param_name.0[0] > #primitive::MAX as u64 {
                    return Err(#down_cast_error.to_string());
                  }

                  return Ok(#param_name.0[0] as #primitive)
                }
              }
              impl #cast_trait_name<#uint> for #primitive {
                fn #cast_function_name(#param_name: #uint)-> #primitive {
                  #primitive::#checked_cast_function_name(#param_name).unwrap()
                }
              }
            })
        });

    // downcast to u64
    if chunk_count >= 2 {
        let overflow_chunks = overflow_chunks(1..chunk_count, param_name.clone());
        let down_cast_error = alloc::format!("Failed to cast {} to u128", uint);
        expanded.append_all(quote! {
          impl #checked_cast_trait_name <#uint> for u64 {
            fn #checked_cast_function_name(#param_name: #uint)-> Result<u64, String> {
              if #overflow_chunks != 0 {
                return Err(#down_cast_error.to_string());
              }

              return Ok(#param_name.low_u64())
            }
          }
          impl #cast_trait_name<#uint> for u64 {
            fn #cast_function_name(#param_name: #uint)-> u64 {
              u64::#checked_cast_function_name(#param_name).unwrap()
            }
          }
        });
    };

    // downcast to u128
    if chunk_count >= 3 {
        let overflow_chunks = overflow_chunks(2..chunk_count, param_name.clone());
        let down_cast_error = alloc::format!("Failed to cast {} to u128", uint);
        expanded.append_all(quote! {
          impl #checked_cast_trait_name <#uint> for u128 {
            fn #checked_cast_function_name(#param_name: #uint)-> Result<u128, String> {
              if #overflow_chunks != 0 {
                return Err(#down_cast_error.to_string());
              }

              return Ok(#param_name.low_u128())
            }
          }
          impl #cast_trait_name<#uint> for u128 {
            fn #cast_function_name(#param_name: #uint)-> u128 {
              u128::#checked_cast_function_name(#param_name).unwrap()
            }
          }
        });
    };
    // downcast from u128
    if chunk_count == 1 {
        let down_cast_error = alloc::format!("Failed to cast u128 to {}", uint);
        expanded.append_all(quote! {
          impl #checked_cast_trait_name <u128> for #uint {
            fn #checked_cast_function_name(#param_name: u128)-> Result<#uint, String> {
              if #param_name > u64::MAX as u128 {
                return Err(#down_cast_error.to_string());
              }

              return Ok(#uint(#param_name as u64))
            }
          }
          impl #cast_trait_name<u128> for #uint {
            fn #cast_function_name(#param_name: u128)-> #uint {
              #uint::#checked_cast_function_name(#param_name).unwrap()
            }
          }
        });
    };
    expanded
}
