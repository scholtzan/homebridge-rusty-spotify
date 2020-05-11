//! Defines the Homebridge Spotify Accessory.

use crate::spotify_api::SpotifyApi;
use js_sys::Array;
use js_sys::Function;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    pub type Service;

    #[wasm_bindgen(method, js_name = getCharacteristic)]
    fn get_characteristic(this: &Service, name: &str) -> Characteristic;

    pub type Characteristic;

    #[wasm_bindgen(method)]
    fn on(this: &Characteristic, event: &str, listener: &Function) -> Characteristic;
}

#[derive(Serialize, Deserialize)]
/// Represents the accessory configuration retrieved from ~/.homebridge/config.json
struct Config {
    /// Spotify API client_id
    pub client_id: String,
    /// Spotify API client_secret
    pub client_secret: String,
    /// Cached refresh token for Spotify API
    pub refresh_token: String,
    /// Device that should be controlled; if None then control currently active device
    pub device_id: Option<String>,
}

#[wasm_bindgen]
/// Represents the Spotify accessory state.
pub struct SpotifyAccessory {
    /// Accessory configuration
    config: Config,
    /// Reference to the Service object
    service: Service,
    /// API to control Spotify
    api: Rc<SpotifyApi>,
    /// Current volume
    volume: Rc<u32>,
}

#[wasm_bindgen]
impl SpotifyAccessory {
    #[wasm_bindgen(constructor)]
    pub fn new(service: Service, _log: Function, config: &JsValue) -> SpotifyAccessory {
        let config: Config = config.into_serde().unwrap();

        let api = SpotifyApi::new(
            config.client_id.clone(),
            config.client_secret.clone(),
            config.refresh_token.clone(),
        );

        SpotifyAccessory {
            config,
            service,
            api: Rc::new(api),
            volume: Rc::new(100),
        }
    }

    /// Return closure returning whether Spotify is currently playing or is paused.
    fn get_on(&self) -> Closure<dyn FnMut(Function)> {
        let api = Rc::clone(&self.api);
        let device_id = self.config.device_id.clone();

        Closure::wrap(Box::new(move |callback: Function| {
            let api = api.clone();
            let device_id = device_id.clone();

            spawn_local(async move {
                let on = match JsFuture::from(api.is_playing(device_id)).await {
                    Ok(state) => state.as_bool().unwrap(),
                    Err(_) => false,
                };

                callback
                    .apply(
                        &JsValue::null(),
                        &Array::of2(&JsValue::null(), &JsValue::from(on)),
                    )
                    .unwrap();
            });
        }) as Box<dyn FnMut(Function)>)
    }

    /// Closure for starting/pausing Spotify.
    fn set_on(&self) -> Closure<dyn FnMut(bool, Function)> {
        let api = Rc::clone(&self.api);
        let device_id = self.config.device_id.clone();

        Closure::wrap(Box::new(move |new_on: bool, callback: Function| {
            if new_on {
                let _ = api.play(device_id.clone());
            } else {
                let _ = api.pause();
            }

            callback
                .apply(
                    &JsValue::null(),
                    &Array::of2(&JsValue::null(), &JsValue::from(new_on)),
                )
                .unwrap();
        }) as Box<dyn FnMut(bool, Function)>)
    }

    /// Returns closure indicating the current volume.
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

    /// Closure for setting the volume.
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
    /// Initializes all service actions.
    pub fn get_services(&self) -> Array {
        let get_on = self.get_on();
        let set_on = self.set_on();

        self.service
            .get_characteristic("On")
            .on("set", set_on.as_ref().unchecked_ref())
            .on("get", get_on.as_ref().unchecked_ref());

        let get_volume = self.get_volume();
        let set_volume = self.set_volume();

        self.service
            .get_characteristic("Brightness")
            .on("set", set_volume.as_ref().unchecked_ref())
            .on("get", get_volume.as_ref().unchecked_ref());

        get_on.forget();
        set_on.forget();
        set_volume.forget();
        get_volume.forget();

        [self.service.clone()].iter().collect()
    }
}
