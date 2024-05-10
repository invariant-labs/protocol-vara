use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::FnArg;
use syn::Type;
pub fn convert_params(input: &syn::ItemFn) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let (conversion_code, converted_params): (Vec<_>, Vec<_>) = input
        .sig
        .inputs
        .iter()
        .map(|param| {
            if let FnArg::Typed(pat_type) = param {
                if let syn::Pat::Ident(ident) = &*pat_type.pat {
                    let param_name = &ident.ident;
                    let js_param_name =
                        syn::Ident::new(&format!("js_{}", param_name), ident.span());
                    let param_ty = &pat_type.ty;

                    let (require_cast, intermediate_cast) = requires_bits_expansion(param_ty);
                    let intermediate_cast = if require_cast {
                        Some(intermediate_cast)
                    } else {
                        None
                    };

                    let conversion_code = if let Some(intermediate_cast) = intermediate_cast {
                        quote! {
                            let #param_name: #intermediate_cast = serde_wasm_bindgen::from_value(#js_param_name)?;
                            let #param_name: #param_ty = #param_name as #param_ty;
                        }
                    } else {
                        quote! {
                            let #param_name: #param_ty = serde_wasm_bindgen::from_value(#js_param_name)?;
                        }
                    };

                    (
                        conversion_code,
                        quote! { #param_name },
                    )
                } else {
                    (quote! {}, quote! {})
                }
            } else {
                (quote! {}, quote! {})
            }
        })
        .unzip();

    (conversion_code, converted_params)
}

pub fn requires_bits_expansion(ty: &Type) -> (bool, syn::Ident) {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            match segment.ident.to_string().as_str() {
                "i32" | "i16" | "i8" => {
                    return (true, syn::Ident::new("i64", segment.ident.span()))
                }
                "u32" | "u16" | "u8" => {
                    return (true, syn::Ident::new("u64", segment.ident.span()))
                }
                _ => return (false, segment.ident.clone()),
            };
        }
    }
    (false, syn::Ident::new("unknown", ty.span()))
}
