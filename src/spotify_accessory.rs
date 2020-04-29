use crate::spotify_api::SpotifyApi;
use js_sys::Array;
use js_sys::Function;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    pub type Switch;

    #[wasm_bindgen(method, js_name = getCharacteristic)]
    fn get_characteristic(this: &Switch, name: &str) -> Characteristic;

    pub type Characteristic;

    #[wasm_bindgen(method)]
    fn on(this: &Characteristic, event: &str, listener: &Function) -> Characteristic;
}

#[derive(Serialize, Deserialize)]
struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    pub device_id: Option<String>,
}

#[wasm_bindgen]
pub struct SpotifyAccessory {
    config: Config,
    on: Rc<bool>,
    service_switch: Switch,
    api: Rc<SpotifyApi>,
    volume: Rc<u32>,
}

#[wasm_bindgen]
impl SpotifyAccessory {
    #[wasm_bindgen(constructor)]
    pub fn new(service_switch: Switch, _log: Function, config: &JsValue) -> SpotifyAccessory {
        let config: Config = config.into_serde().unwrap();

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
            volume: Rc::new(100),
        }
    }

    fn get_on(&self) -> Closure<dyn FnMut(Function)> {
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
    }

    fn set_on(&self) -> Closure<dyn FnMut(bool, Function)> {
        let mut on = Rc::clone(&self.on);
        let api = Rc::clone(&self.api);
        let device_id = self.config.device_id.clone();

        Closure::wrap(Box::new(move |new_on: bool, callback: Function| {
            on = Rc::new(new_on);

            if new_on {
                let _ = api.play(device_id.clone());
            } else {
                let _ = api.pause();
            }

            callback
                .apply(
                    &JsValue::null(),
                    &Array::of2(&JsValue::null(), &JsValue::from(*on)),
                )
                .unwrap();
        }) as Box<dyn FnMut(bool, Function)>)
    }

    fn get_volume(&self) -> Closure<dyn FnMut(Function)> {
        let volume = Rc::clone(&self.volume);

        Closure::wrap(Box::new(move |callback: Function| {
            let volume = *volume;

            callback
                .apply(
                    &JsValue::null(),
                    &Array::of2(&JsValue::null(), &JsValue::from(volume)),
                )
                .unwrap();
        }) as Box<dyn FnMut(Function)>)
    }

    fn set_volume(&self) -> Closure<dyn FnMut(u32, Function)> {
        let api = Rc::clone(&self.api);
        let mut volume = Rc::clone(&self.volume);

        Closure::wrap(Box::new(move |new_volume: u32, callback: Function| {
            volume = Rc::new(new_volume);
            let _ = api.volume(*volume);

            callback
                .apply(
                    &JsValue::null(),
                    &Array::of2(&JsValue::null(), &JsValue::from(*volume)),
                )
                .unwrap();
        }) as Box<dyn FnMut(u32, Function)>)
    }

    #[wasm_bindgen(js_name = getServices)]
    pub fn get_services(&self) -> Array {
        let get_on = self.get_on();
        let set_on = self.set_on();

        self.service_switch
            .get_characteristic("On")
            .on("set", set_on.as_ref().unchecked_ref())
            .on("get", get_on.as_ref().unchecked_ref());

        let get_volume = self.get_volume();
        let set_volume = self.set_volume();

        self.service_switch
            .get_characteristic("Brightness")
            .on("set", set_volume.as_ref().unchecked_ref())
            .on("get", get_volume.as_ref().unchecked_ref());

        get_on.forget();
        set_on.forget();
        set_volume.forget();
        get_volume.forget();

        [self.service_switch.clone()].iter().collect()
    }
}
