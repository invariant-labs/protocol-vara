use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, Ident};

pub fn process_params(input: &syn::ItemFn) -> Vec<TokenStream> {
    input
        .sig
        .inputs
        .iter()
        .filter_map(|param| {
            if let FnArg::Typed(pat_type) = param {
                if let syn::Pat::Ident(ident) = &*pat_type.pat {
                    let param_name = &ident.ident;
                    let js_param_name =
                        syn::Ident::new(&format!("js_{}", param_name), ident.span());
                    let param_ty = quote! { wasm_bindgen::JsValue };
                    Some(quote! { #js_param_name: #param_ty })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

pub fn process_return_type(
    return_ty: &proc_macro2::TokenStream,
    camel_case_string: &String,
) -> (Ident, Vec<proc_macro2::TokenStream>, bool) {
    if !is_result_wrapped(&return_ty) {
        return (
            Ident::new("unknown", proc_macro2::Span::call_site()),
            Vec::new(),
            true,
        );
    }

    let field_names = collect_field_names(return_ty);

    if field_names.len() == 0 {
        return (
            Ident::new("unknown", proc_macro2::Span::call_site()),
            Vec::new(),
            false,
        );
    }

    let (tuple_struct_name, tuple_struct_fields) =
        construct_tuple_struct(&field_names, &camel_case_string);

    (tuple_struct_name, tuple_struct_fields, false)
}

pub fn is_result_wrapped(return_ty: &proc_macro2::TokenStream) -> bool {
    match return_ty
        .clone()
        .into_iter()
        .next()
        .unwrap()
        .to_string()
        .as_str()
    {
        "TrackableResult" => true,
        "Result" => true,
        _ => false,
    }
}

pub fn collect_field_names(return_ty: &proc_macro2::TokenStream) -> Vec<String> {
    let mut field_names: Vec<String> = Vec::new();

    for token in return_ty.clone().into_iter() {
        match token {
            TokenTree::Group(group) => {
                for inner_token in group.stream() {
                    match inner_token {
                        TokenTree::Ident(ident) => {
                            field_names.push(ident.to_string());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    field_names
}

pub fn construct_tuple_struct(
    field_names: &Vec<String>,
    camel_case_string: &String,
) -> (Ident, Vec<proc_macro2::TokenStream>) {
    let tuple_struct_name = Ident::new(
        &format!("{}{}", camel_case_string, "Result"),
        proc_macro2::Span::call_site(),
    );
    let tuple_struct_fields: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .map(|ident| {
            let field_ident = Ident::new(ident, proc_macro2::Span::call_site());
            quote::quote! { #field_ident }
        })
        .collect();
    (tuple_struct_name, tuple_struct_fields)
}

pub fn construct_camel_case(args: &Vec<&str>, original_function_name: String) -> String {
    let camel_case_string = if args.len() == 1 && !args[0].is_empty() {
        let trimmed_string = args[0].trim_matches(|c| c == '"' || c == '\\');
        trimmed_string.to_string()
    } else {
        let camel_case: String = original_function_name
            .chars()
            .scan(false, |capitalize, ch| {
                if ch == '_' {
                    *capitalize = true;
                    Some(None)
                } else {
                    if *capitalize {
                        *capitalize = false;
                        Some(Some(ch.to_ascii_uppercase()))
                    } else {
                        Some(Some(ch))
                    }
                }
            })
            .flatten()
            .collect();
        camel_case
    };
    camel_case_string
}

pub fn construct_fn_ident(fn_name: &syn::Ident) -> syn::Ident {
    let generated_function_name = format!("wrapped_{}", fn_name);
    syn::Ident::new(&generated_function_name, fn_name.span())
}
