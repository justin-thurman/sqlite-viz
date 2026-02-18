mod model;
mod parser;
mod utils;

use crate::parser::parse_database;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// extern {
//     fn alert(s: &str);
// }

#[wasm_bindgen]
pub fn analyze_db(bytes: &[u8]) -> Result<JsValue, JsError> {
    match parse_database(bytes) {
        Ok(result) => Ok(serde_wasm_bindgen::to_value(&result)?),
        Err(e) => Err(JsError::from(&*e)),
    }
}
