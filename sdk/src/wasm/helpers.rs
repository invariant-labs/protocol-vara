use wasm_bindgen::prelude::*;

// Logging to typescript
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_many(a: &str, b: &str);
}
// run once in js for debugging purposes
#[wasm_bindgen]
pub fn init_panic_hook() {
  console_error_panic_hook::set_once();
}

#[macro_export]
macro_rules! decimal_ops {
    ($decimal:ident) => {
        ::paste::paste! {
            #[wasm_bindgen]
            #[allow(non_snake_case)]
            pub fn [<get $decimal Scale >] () -> BigInt {
                BigInt::from($decimal::scale())
            }

            #[wasm_bindgen]
            #[allow(non_snake_case)]
            pub fn [<get $decimal Denominator >] () -> BigInt {
                // should be enough for current denominators
                BigInt::from($decimal::from_integer(1).get().as_u128())
            }

            #[wasm_bindgen]
            #[allow(non_snake_case)]
            pub fn [<to $decimal >] (js_val: JsValue, js_scale: JsValue) -> BigInt {
                let js_val: u64 = convert!(js_val).unwrap();
                let scale: u64 = convert!(js_scale).unwrap();
                $decimal::from_scale(js_val, scale as u8)
                .get().0
                .iter().rev()
                .fold(BigInt::from(0), |acc, &x| (acc << BigInt::from(64)) | BigInt::from(x))
            }
        }
    };
}

#[macro_export]
macro_rules! convert {
    ($value:expr) => {{
        serde_wasm_bindgen::from_value($value)
    }};
}

#[macro_export]
macro_rules! resolve {
    ($result:expr) => {{
        match $result {
            Ok(value) => Ok(serde_wasm_bindgen::to_value(&value)?),
            Err(error) => Err(JsValue::from_str(&error.to_string())),
        }
    }};
}
