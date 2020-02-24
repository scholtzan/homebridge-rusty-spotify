use wasm_bindgen::prelude::*;
use web_sys::console;
use js_sys::Function;

#[wasm_bindgen]
pub struct SamplePlatform {
    log: Function,
    config: JsValue
}

#[wasm_bindgen]
impl SamplePlatform {
    #[wasm_bindgen(constructor)]
    pub fn new(log: Function, config: JsValue) -> SamplePlatform {
        console::log_1(&"Hello using web-sys".into());

        SamplePlatform {
            log,
            config
        }
    }
}