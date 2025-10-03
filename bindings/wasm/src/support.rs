// Support utilities for WASM bindings
use serde::de::DeserializeOwned;

/// Parse JSON config with defaults
/// Returns the deserialized config or the default value if parsing fails or config is empty
pub fn parse_with_defaults<T: DeserializeOwned + Default>(config_json: &str) -> T {
    if config_json.trim().is_empty() || config_json == "{}" {
        T::default()
    } else {
        serde_json::from_str::<T>(config_json).unwrap_or_else(|_| T::default())
    }
}

/// Macro to generate a WASM function wrapper that calls a core function with parsed config
/// 
/// Usage:
/// ```
/// wasm_fn! {
///     pub fn function_name(text: &str, config_json: &str) -> Result<OutputType, JsValue> {
///         core_function(text, &ConfigType) -> Result<Vec<Element>, String>
///     }
/// }
/// ```
#[macro_export]
macro_rules! wasm_fn {
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($text:ident: &str, $config:ident: &str) 
        -> Result<$result:ty, JsValue>
        with $core_fn:path, $config_type:ty, $result_wrapper:expr
    ) => {
        #[wasm_bindgen]
        $(#[$meta])*
        $vis fn $name($text: &str, $config: &str) -> Result<$result, JsValue> {
            let params = $crate::support::parse_with_defaults::<$config_type>($config);
            $core_fn($text, &params)
                .map($result_wrapper)
                .map_err(|e| JsValue::from_str(&e))
        }
    };
}
