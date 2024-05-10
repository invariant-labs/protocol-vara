use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn generate_exported_function(
    tuple_struct_name: &Ident,
    tuple_struct_fields: Vec<TokenStream>,
    camel_case_function_name: &str,
    generated_function_ident: &Ident,
    params: &Vec<TokenStream>,
    conversion_code: &Vec<TokenStream>,
    converted_params: &Vec<TokenStream>,
    original_function_name: &Ident,
    result_not_wrapped: bool,
    return_type: String,
) -> TokenStream {
    if tuple_struct_fields.len() > 0 {
        tuple_exported_function(
            &tuple_struct_name,
            tuple_struct_fields,
            &camel_case_function_name,
            &generated_function_ident,
            &params,
            &conversion_code,
            &converted_params,
            &original_function_name,
        )
    } else if result_not_wrapped {
        value_exported_function(
            &camel_case_function_name,
            &generated_function_ident,
            &params,
            &conversion_code,
            &converted_params,
            &original_function_name,
            return_type,
        )
    } else {
        struct_exported_function(
            &camel_case_function_name,
            &generated_function_ident,
            &params,
            &conversion_code,
            &converted_params,
            &original_function_name,
        )
    }
}

pub fn value_exported_function(
    camel_case_function_name: &str,
    generated_function_ident: &Ident,
    params: &Vec<TokenStream>,
    conversion_code: &Vec<TokenStream>,
    converted_params: &Vec<TokenStream>,
    original_function_name: &Ident,
    return_type: String,
) -> TokenStream {
    let result_conversion = match return_type.as_str() {
        "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" | "bool" => {
            quote! {
                BigInt::from(result)
            }
        }
        _ => {
            quote! {
                BigInt::from(result.get())
            }
        }
    };
    quote! {
        #[wasm_bindgen(js_name = #camel_case_function_name)]
        pub fn #generated_function_ident(#(#params),*) -> Result<BigInt, JsValue> {
            #(#conversion_code)*

            let result = #original_function_name(#(#converted_params),*);
            Ok(#result_conversion)
        }
    }
}

pub fn tuple_exported_function(
    tuple_struct_name: &Ident,
    tuple_struct_fields: Vec<TokenStream>,
    camel_case_function_name: &str,
    generated_function_ident: &Ident,
    params: &Vec<TokenStream>,
    conversion_code: &Vec<TokenStream>,
    converted_params: &Vec<TokenStream>,
    original_function_name: &Ident,
) -> TokenStream {
    let tuple_struct_instance = {
        let fields: Vec<_> = tuple_struct_fields
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let index = syn::Index::from(i);
                quote! { tuple.#index }
            })
            .collect();

        quote! { #tuple_struct_name(#(#fields),*) }
    };

    quote! {
        #[derive(serde::Serialize, serde::Deserialize, Tsify)]
        #[tsify(into_wasm_abi, from_wasm_abi)]
        pub struct #tuple_struct_name (
            #(#tuple_struct_fields),*
        );

        #[wasm_bindgen(js_name = #camel_case_function_name)]
        pub fn #generated_function_ident(#(#params),*) -> Result<JsValue, JsValue> {
            #(#conversion_code)*

            let result = #original_function_name(#(#converted_params),*);

            match result {
                Ok(tuple) => {
                    let mut tuple_struct_instance = #tuple_struct_instance;

                    Ok(serde_wasm_bindgen::to_value(&tuple_struct_instance)?)
                }
                Err(error) => Err(JsValue::from_str(&error.to_string())),
            }
        }
    }
}

pub fn struct_exported_function(
    camel_case_function_name: &str,
    generated_function_ident: &Ident,
    params: &Vec<TokenStream>,
    conversion_code: &Vec<TokenStream>,
    converted_params: &Vec<TokenStream>,
    original_function_name: &Ident,
) -> TokenStream {
    quote! {
        #[wasm_bindgen(js_name = #camel_case_function_name)]
        pub fn #generated_function_ident(#(#params),*) -> Result<JsValue, JsValue> {
            #(#conversion_code)*

            let result = #original_function_name(#(#converted_params),*);

            match result {
                Ok(v) => Ok(serde_wasm_bindgen::to_value(&v)?),
                Err(error) => Err(JsValue::from_str(&error.to_string())),
            }
        }
    }
}
