use crate::spotify_api::SpotifyApi;
use js_sys::Array;
use js_sys::Function;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const MAX_VOLUME: u32 = 100;
const VOLUME_INCREASE: u32 = 10;

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

    #[wasm_bindgen(js_name = setInterval)]
    pub fn set_interval(closure: &Function, millis: u32) -> JsValue;

    #[wasm_bindgen(js_name = clearInterval)]
    pub fn clear_interval(id: &JsValue);
}

#[derive(Serialize, Deserialize)]
struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    pub fade: Option<u32>,
    pub device_id: Option<String>,
}

#[wasm_bindgen]
pub struct SpotifyAccessory {
    config: Config,
    on: Rc<bool>,
    service_switch: Switch,
    api: Rc<SpotifyApi>,
    volume: Rc<u32>,
    volume_interval: Rc<JsValue>,
}

#[wasm_bindgen]
impl SpotifyAccessory {
    #[wasm_bindgen(constructor)]
    pub fn new(service_switch: Switch, _log: Function, config: &JsValue) -> SpotifyAccessory {
        let config: Config = config.into_serde().unwrap();
        // config: https://rustwasm.github.io/docs/wasm-bindgen/reference/accessing-properties-of-untyped-js-values.html

        let api = SpotifyApi::new(
            config.client_id.clone(),
            config.client_secret.clone(),
            config.refresh_token.clone(),
        );

        SpotifyAccessory {
            config,
            on: Rc::new(false),
            service_switch,
            api: Rc::new(api),
            volume: Rc::new(MAX_VOLUME),
            volume_interval: Rc::new(JsValue::NULL),
        }
    }

    #[wasm_bindgen(js_name = getServices)]
    pub fn get_services(&self) -> Array {
        let get_on = {
            let on = Rc::clone(&self.on);

            Closure::wrap(Box::new(move |callback: Function| {
                let on = *on;

                callback
                    .apply(
                        &JsValue::null(),
                        &Array::of2(&JsValue::null(), &JsValue::from(on)),
                    )
                    .unwrap();
            }) as Box<dyn FnMut(Function)>)
        };

        let set_on = {
            let mut on = Rc::clone(&self.on);
            let api = Rc::clone(&self.api);
            let mut volume = Rc::clone(&self.volume);
            let device_id = self.config.device_id.clone();
            let fade = self.config.fade.clone();
            let mut volume_interval = Rc::clone(&self.volume_interval);

            Closure::wrap(Box::new(move |new_on: bool, callback: Function| {
                on = Rc::new(new_on);

                if new_on {
                    if let Some(fade) = fade {
                        if fade == 0 {
                            return;
                        }

                        volume = Rc::new(0);
                        let _ = api.volume(*volume);
                        let mut volume = Rc::clone(&volume);
                        let api = Rc::clone(&api);
                        let token = Rc::clone(&volume_interval);

                        let volume_closure = Closure::wrap(Box::new(move || {
                            volume = Rc::new(*volume + VOLUME_INCREASE);
                            let _ = api.volume(*volume);

                            if *volume >= MAX_VOLUME {
                                clear_interval(&(*token));
                            }
                        })
                            as Box<dyn FnMut()>);

                        let time_interval = fade / (MAX_VOLUME / VOLUME_INCREASE) * 1000;

                        clear_interval(&(*volume_interval));

                        let token =
                            set_interval(volume_closure.as_ref().unchecked_ref(), time_interval);
                        volume_interval = Rc::new(token);

                        volume_closure.forget();
                    }

                    let _ = api.play(device_id.clone());
                } else {
                    clear_interval(&(*volume_interval));
                    let _ = api.pause();
                }

                callback
                    .apply(
                        &JsValue::null(),
                        &Array::of2(&JsValue::null(), &JsValue::from(*on)),
                    )
                    .unwrap();
            }) as Box<dyn FnMut(bool, Function)>)
        };

        self.service_switch
            .get_characteristic("On")
            .on("set", set_on.as_ref().unchecked_ref())
            .on("get", get_on.as_ref().unchecked_ref());

        get_on.forget();
        set_on.forget();

        [self.service_switch.clone()].iter().collect()
    }
}
