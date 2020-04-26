use wasm_bindgen::prelude::*;
use web_sys::{console, Request, RequestInit, RequestMode, Response};
use js_sys::Function;
use js_sys::Array;
use std::cell::RefCell;
use std::rc::Rc;
use std::future::Future;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::{spawn_local, future_to_promise};
use base64::encode;
use crate::spotify_api::SpotifyApi;

#[wasm_bindgen]
extern "C" {
    pub type Switch;

    #[wasm_bindgen(method, js_name = getCharacteristic)]
    fn get_characteristic(this: &Switch, name: &str) -> Characteristic;

    type Characteristic;

    #[wasm_bindgen(method)]
    fn on(this: &Characteristic, event: &str, listener: &Function) -> Characteristic;

    #[wasm_bindgen(extends = Characteristic)]
    type On;

    #[wasm_bindgen(constructor)]
    fn init_on() -> On;

    #[wasm_bindgen(method, js_name = On)]
    fn get_on(this: &Characteristic) -> On;
}



#[derive(Serialize, Deserialize)]
struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String
}

#[wasm_bindgen]
pub struct SpotifyAccessory {
    log: Function,
    config: Config,
    on: Rc<bool>,
    service_switch: Switch,
    api: Rc<SpotifyApi>
}

#[wasm_bindgen]
impl SpotifyAccessory {
    #[wasm_bindgen(constructor)]
    pub fn new(service_switch: Switch, log: Function, config: &JsValue) -> SpotifyAccessory {
        console::log_1(&"Hello using web-sys".into());

        let config: Config = config.into_serde().unwrap();
        // config: https://rustwasm.github.io/docs/wasm-bindgen/reference/accessing-properties-of-untyped-js-values.html

        let api = SpotifyApi::new(
            config.client_id.clone(),
            config.client_secret.clone(),
            config.refresh_token.clone()
        );

        SpotifyAccessory {
            log,
            config,
            on: Rc::new(false),
            service_switch,
            api: Rc::new(api)
        }
    }

    fn stop(&self){
        console::log_1(&"stop music".into());
    }

    fn start(&self) -> Box<dyn Fn()> {
        return Box::new(move || {
            console::log_1(&"start music".into());
        })
    }

    #[wasm_bindgen(js_name = getServices)]
    pub fn get_services(&self) -> Array {
        console::log_1(&"get Spotify service 1".into());

        let get_on = {
            let on = Rc::clone(&self.on);
            let api = Rc::clone(&self.api);

            Closure::wrap(Box::new(move |callback: Function| {
                console::log_1(&"get on".into());

                let pause_request = api.pause();

                spawn_local(async move {
                    JsFuture::from(pause_request).await.unwrap().as_string().unwrap();
                });

                callback.apply(&JsValue::null(), &Array::of2(&JsValue::null(), &JsValue::from(*on))).unwrap();
            }) as Box<dyn FnMut(Function)>)
        };

        let set_on = {
            let mut on = Rc::clone(&self.on);
            let api = Rc::clone(&self.api);

            Closure::wrap(Box::new(move |new_on: bool, callback: Function| {
                console::log_1(&"set on".into());
                on = Rc::new(new_on);
                let play_request = api.play();

                spawn_local(async move {
                    JsFuture::from(play_request).await.unwrap().as_string().unwrap();
                });

                callback.apply(&JsValue::null(), &Array::of2(&JsValue::null(), &JsValue::from(*on))).unwrap();
            }) as Box<dyn FnMut(bool, Function)>)
        };


        // https://stackoverflow.com/questions/53214434/how-to-return-a-rust-closure-to-javascript-via-webassembly
        let c = self.service_switch.get_characteristic("On");
        c.on("set", set_on.as_ref().unchecked_ref()).on("get", get_on.as_ref().unchecked_ref());

        get_on.forget();
        set_on.forget();

        console::log_1(&"get Spotify service".into());

        [self.service_switch.clone()].iter().collect()
    }
}