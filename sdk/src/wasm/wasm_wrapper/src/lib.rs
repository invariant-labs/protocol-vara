extern crate proc_macro;

mod conversion;
mod export;
mod helpers;
use crate::conversion::convert_params;
use crate::export::generate_exported_function;
use crate::helpers::{
    construct_camel_case, construct_fn_ident, process_params, process_return_type,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn wasm_wrapper(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let args_str = attr.to_string();
    let args: Vec<&str> = args_str.split(',').collect();

    let original_function_name = &input.sig.ident;
    let camel_case_function_name = construct_camel_case(&args, original_function_name.to_string());
    let generated_function_ident = construct_fn_ident(original_function_name);

    let return_ty = match &input.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };
    let return_type_name = return_ty.clone().into_iter().next().unwrap().to_string();
    let (tuple_struct_name, tuple_struct_fields, result_not_wrapped) =
        process_return_type(&return_ty, &camel_case_function_name);

    let params: Vec<_> = process_params(&input);

    let (conversion_code, converted_params): (Vec<_>, Vec<_>) = convert_params(&input);

    let exported_function = generate_exported_function(
        &tuple_struct_name,
        tuple_struct_fields,
        &camel_case_function_name,
        &generated_function_ident,
        &params,
        &conversion_code,
        &converted_params,
        &original_function_name,
        result_not_wrapped,
        return_type_name,
    );

    let result = quote! {
        #input
        #exported_function
    };

    result.into()
}
