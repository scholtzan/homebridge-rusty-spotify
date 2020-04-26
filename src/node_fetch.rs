use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use js_sys::{Array, Function, Promise};
use web_sys::{console};
use std::collections::HashMap;
use futures::executor::block_on;
use futures::Future;
use std::rc::Rc;

#[wasm_bindgen]
extern "C" {
    fn require(name: &str) -> Function;

    type Response;

    #[wasm_bindgen(method, js_name = json)]
    fn json(this: &Response) -> Promise;

    #[wasm_bindgen(method, js_name = text)]
    fn text(this: &Response) -> Promise;
}

pub enum FetchMethod {
    Get,
    Post,
    Put,
    Delete
}

impl FetchMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            &FetchMethod::Get => "GET",
            &FetchMethod::Post => "POST",
            &FetchMethod::Put => "PUT",
            &FetchMethod::Delete => "DELETE"
        }
    }
}

#[derive(Serialize)]
struct RequestOptions {
    method: String,
    body: String,
    headers: HashMap<String, String>
}


pub async fn fetch(url: &str, method: FetchMethod, body: &str, headers: HashMap<String, String>, empty_response: bool) -> Result<JsValue, JsValue> {
    let fetch = require("node-fetch");
    let options = RequestOptions {
        method: method.as_str().to_owned(),
        body: body.to_owned(),
        headers
    };

    console::log_1(&format!("options {:?}", &JsValue::from_serde(&options).unwrap()).into());

    let fetch_result = fetch.apply(&JsValue::null(), &Array::of2(&JsValue::from(url), &JsValue::from_serde(&options).unwrap()));
    match fetch_result {
        Ok(p) => {
            let promise = Promise::from(p);
            let resp_value = JsFuture::from(promise).await?;
            let resp: Response = resp_value.unchecked_into();

            if empty_response {
                Ok(JsValue::NULL)
            } else {
                let json: JsValue = JsFuture::from(resp.json()).await?;
                console::log_1(&format!("JSON {:?}", json).into());

                Ok(json)
            }
        },
        _ => {
            console::log_1(&"Error executing fetch request".into());
            Err(JsValue::from("Error executing fetch request"))
        }
    }
}


