//! Defines the Homebridge Spotify Accessory.

use crate::spotify_api::SpotifyApi;
use js_sys::Array;
use js_sys::Function;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;

use crate::spotify_platform::{Service, ServiceType};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(method, js_name = getCharacteristic)]
    fn get_characteristic(this: &Service, name: &str) -> Characteristic;

    pub type Characteristic;

    #[wasm_bindgen(method)]
    fn on(this: &Characteristic, event: &str, listener: &Function) -> Characteristic;

    #[wasm_bindgen(method, js_name = setValue)]
    fn set_value(this: &Characteristic, value: &str);

    #[wasm_bindgen(method, js_name = getValue)]
    fn get_value(this: &Characteristic);

    #[derive(Debug, PartialEq)]
    pub type Accessory;

    #[wasm_bindgen(constructor, js_class = "Accessory")]
    fn new(name: &str, uuid: &str) -> Accessory;

    #[wasm_bindgen(method, js_name = addService)]
    fn add_service(this: &Accessory, service: &Service);

    #[wasm_bindgen(method, getter = UUID)]
    pub fn get_uuid(this: &Accessory) -> String;

    pub type UUIDGen;

    #[wasm_bindgen(static_method_of = UUIDGen)]
    fn generate(uuid_base: &str) -> String;

    #[wasm_bindgen(js_name = createLight)]
    pub fn create_light(name: &str) -> Service;

    #[wasm_bindgen(js_name = createSpeaker)]
    pub fn create_speaker(name: &str) -> Service;
}

#[wasm_bindgen]
#[derive(Debug)]
/// Represents the Spotify accessory state.
pub struct SpotifyAccessory {
    /// Reference to the Service object
    service: Service,
    /// API to control Spotify
    api: Rc<SpotifyApi>,
    /// ID of device to be controlled
    device_id: String,
    /// Accessory display name
    name: String,
    /// Accessory to be registered to Homebridge
    accessory: Accessory,
    /// Homebridge service type
    service_type: ServiceType,
}

impl SpotifyAccessory {
    pub fn new(
        name: String,
        device_id: String,
        service_type: ServiceType,
        api: Rc<SpotifyApi>,
    ) -> SpotifyAccessory {
        // accessory type that can get registered to Homebridge
        let accessory = Accessory::new(&name, &UUIDGen::generate(&name));

        let service = match service_type {
            ServiceType::Light => create_light(&name),
            ServiceType::Speaker => create_speaker(&name),
        };

        let spotify_accessory = SpotifyAccessory {
            service,
            api,
            device_id,
            name,
            accessory,
            service_type,
        };

        spotify_accessory.apply_characteristics();
        spotify_accessory.apply_service();

        spotify_accessory
    }

    /// Return the Spotify device ID.
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }

    /// Return the Homebridge accessory.
    pub fn get_accessory(&self) -> &Accessory {
        &self.accessory
    }

    /// Manually check if the device is playing.
    /// Homekit only checks when the app gets opened, so if the
    /// status changes while the app is open, the status is not
    /// reflected correctly.
    pub fn check_on(&self) {
        self.service.get_characteristic("On").get_value();
    }

    /// Setup up Homebridge characteristics.
    fn apply_characteristics(&self) {
        let get_on = self.get_on();
        let set_on = self.set_on();

        match self.service_type {
            ServiceType::Light => self
                .service
                .get_characteristic("On")
                .on("set", set_on.as_ref().unchecked_ref())
                .on("get", get_on.as_ref().unchecked_ref()),
            ServiceType::Speaker => self
                .service
                .get_characteristic("Mute")
                .on("set", set_on.as_ref().unchecked_ref())
                .on("get", get_on.as_ref().unchecked_ref()),
        };

        let get_volume = self.get_volume();
        let set_volume = self.set_volume();

        match self.service_type {
            ServiceType::Light => self
                .service
                .get_characteristic("Brightness")
                .on("set", set_volume.as_ref().unchecked_ref())
                .on("get", get_volume.as_ref().unchecked_ref()),
            ServiceType::Speaker => self
                .service
                .get_characteristic("Volume")
                .on("set", set_volume.as_ref().unchecked_ref())
                .on("get", get_volume.as_ref().unchecked_ref()),
        };

        self.service
            .get_characteristic("Name")
            .set_value(&self.name);

        get_on.forget();
        set_on.forget();
        set_volume.forget();
        get_volume.forget();
    }

    /// Connect service with accessory.
    fn apply_service(&self) {
        self.accessory.add_service(&self.service)
    }

    /// Return closure returning whether Spotify is currently playing or is paused.
    fn get_on(&self) -> Closure<dyn FnMut(Function)> {
        let api = Rc::clone(&self.api);
        let device_id = self.device_id.clone();

        Closure::wrap(Box::new(move |callback: Function| {
            let api = api.clone();
            let device_id = device_id.clone();

            spawn_local(async move {
                let on = match JsFuture::from(api.is_playing(device_id)).await {
                    Ok(state) => state.as_bool().unwrap_or(false),
                    Err(_) => false,
                };

                callback
                    .apply(
                        &JsValue::null(),
                        &Array::of2(&JsValue::null(), &JsValue::from(on)),
                    )
                    .ok();
            });
        }) as Box<dyn FnMut(Function)>)
    }

    /// Closure for starting/pausing Spotify.
    fn set_on(&self) -> Closure<dyn FnMut(bool, Function)> {
        let api = Rc::clone(&self.api);
        let device_id = self.device_id.clone();
        let service_type = self.service_type.clone();

        Closure::wrap(Box::new(move |new_on: bool, callback: Function| {
            match service_type {
                ServiceType::Light => {
                    if new_on {
                        let _ = api.play(device_id.clone());
                    } else {
                        let _ = api.pause(device_id.clone());
                    }
                }
                ServiceType::Speaker => {
                    // speaker uses the Mute characteristic which is basically
                    // the inverse of on/off for play/pause
                    // Mute on == pause
                    // Mute off == play
                    if new_on {
                        let _ = api.pause(device_id.clone());
                    } else {
                        let _ = api.play(device_id.clone());
                    }
                }
            }

            callback
                .apply(
                    &JsValue::null(),
                    &Array::of2(&JsValue::null(), &JsValue::from(new_on)),
                )
                .ok();
        }) as Box<dyn FnMut(bool, Function)>)
    }

    /// Returns closure indicating the current volume.
    fn get_volume(&self) -> Closure<dyn FnMut(Function)> {
        let api = Rc::clone(&self.api);
        let device_id = self.device_id.clone();

        Closure::wrap(Box::new(move |callback: Function| {
            let api = api.clone();
            let device_id = device_id.clone();

            spawn_local(async move {
                let volume: u32 = match JsFuture::from(api.get_volume(device_id)).await {
                    Ok(state) => (state.as_f64().unwrap_or(50.0) as u32),
                    Err(_) => 50,
                };

                callback
                    .apply(
                        &JsValue::null(),
                        &Array::of2(&JsValue::null(), &JsValue::from(volume)),
                    )
                    .ok();
            });
        }) as Box<dyn FnMut(Function)>)
    }

    /// Closure for setting the volume.
    fn set_volume(&self) -> Closure<dyn FnMut(u32, Function)> {
        let api = Rc::clone(&self.api);
        let device_id = self.device_id.clone();

        Closure::wrap(Box::new(move |new_volume: u32, callback: Function| {
            let _ = api.set_volume(device_id.clone(), new_volume);

            callback
                .apply(
                    &JsValue::null(),
                    &Array::of2(&JsValue::null(), &JsValue::from(new_volume)),
                )
                .ok();
        }) as Box<dyn FnMut(u32, Function)>)
    }
}
