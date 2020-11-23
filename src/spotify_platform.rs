//! Defines the Homebridge Spotify Platform.

use crate::spotify_api::SpotifyApi;
use js_sys::{Array, Function, Object};
use std::rc::Rc;
use std::cell::RefCell;
use std::env;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

use crate::spotify_accessory::SpotifyAccessory;
use crate::spotify_accessory::Accessory;
use crate::spotify_api::SpotifyDevicesResponse;

const REFRESH_RATE: u32 = 10 * 1000; // milliseconds, todo: configure
const PLUGIN_IDENTIFIER: &str = "homebridge-rusty-spotify";
const PLATFORM_NAME: &str = "Spotify";

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type Service;

    #[derive(Clone)]
    pub type Homebridge;

    #[derive(Clone, Debug)]
    #[wasm_bindgen(js_name = Array)]
    pub type PlatformAccessories;

    #[wasm_bindgen(constructor, js_class = "Array")]
    fn of(accessory: &Accessory) -> PlatformAccessories;

    #[wasm_bindgen(method, js_name = registerPlatformAccessories)]
    fn register_platform_accessories(this: &Homebridge, plugin_identifier: &str, platform_name: &str, accessories: PlatformAccessories);

    #[wasm_bindgen(method, js_name = unregisterPlatformAccessories)]
    fn unregister_platform_accessories(this: &Homebridge, plugin_identifier: &str, platform_name: &str, accessories: PlatformAccessories);

    #[wasm_bindgen(method)]
    fn on(this: &Homebridge, event: &str, listener: &Function);

    #[wasm_bindgen(js_name = setInterval)]
    pub fn set_interval(closure: &Function, millis: u32) -> f64;
}

#[derive(Serialize, Deserialize)]
/// Represents the platform configuration retrieved from ~/.homebridge/config.json
struct Config {
    /// Spotify API client_id
    pub client_id: String,
    /// Spotify API client_secret
    pub client_secret: String,
    /// Cached refresh token for Spotify API
    pub refresh_token: String,
}


#[wasm_bindgen]
/// Represents the Spotify accessory state.
pub struct SpotifyPlatform {
    /// Homebridge reference
    homebridge: Homebridge,
    /// Platform configuration
    config: Config,
    /// API to control Spotify
    api: Rc<SpotifyApi>,
    /// Available Spotify devices
    devices: Rc<RefCell<Vec<SpotifyAccessory>>>,
    /// Cached accessories
    cached: Rc<RefCell<Vec<Accessory>>>,

    #[wasm_bindgen(js_name = pollingInterval)]
    pub polling_interval: i64
}

#[wasm_bindgen]
impl SpotifyPlatform {
    #[wasm_bindgen(constructor)]
    pub fn new(homebridge: Homebridge, _log: Function, config: &JsValue) -> SpotifyPlatform {
        let config: Config = config.into_serde().unwrap();

        let api = SpotifyApi::new(
            config.client_id.clone(),
            config.client_secret.clone(),
            config.refresh_token.clone(),
        );

        let mut platform = SpotifyPlatform {
            homebridge,
            config,
            api: Rc::new(api),
            devices: Rc::new(RefCell::new(Vec::new())),
            polling_interval: 1000,
            cached: Rc::new(RefCell::new(Vec::new()))
        };

        platform.refresh_devices();
        platform
    }

    /// Determine available Spotify devices and add them as accessories,
    /// remove devices that became inactive.
    fn refresh_devices(&mut self) {
        let homebridge = self.homebridge.clone();
        let api = self.api.clone();
        let cached = self.cached.clone();
        let devices = self.devices.clone();

        let refresh_closure = Closure::wrap(Box::new(move || {
            let homebridge = homebridge.clone();
            let api = api.clone();
            let cached = cached.clone();
            let devices = devices.clone();

            console::log_1(&"Check device statuses".into());

            spawn_local(async move {
                console::log_1(&"Do check".into());

                Self::remove_cached(&homebridge, cached);

                let available_devices: SpotifyDevicesResponse = match JsFuture::from(api.get_devices()).await {
                    Ok(state) => state.into_serde().unwrap(),
                    Err(_) => SpotifyDevicesResponse { devices: Vec::new() },
                };

                // check if devices still exist
                devices.borrow_mut().retain(|registered_device| {
                    if !available_devices.devices.iter().any(|d| &d.id == registered_device.get_device_id()) {
                        let accessories = PlatformAccessories::of(registered_device.get_accessory());

                        console::log_1(&format!("Unregister Spotify device: {:?}", registered_device.get_accessory()).into());

                        homebridge.unregister_platform_accessories(
                            PLUGIN_IDENTIFIER,
                            PLATFORM_NAME,
                            accessories
                        );

                        return false;
                    }
                    true
                });

                // check if device already exists, otherwise add
                for available_device in available_devices.devices {
                    if !devices.borrow().iter().any(|d| d.get_device_id() == &available_device.id) {
                        let accessory = SpotifyAccessory::new(
                            available_device.name,
                            available_device.id,
                            api.clone());

                        console::log_1(&format!("Register Spotify device: {:?}", accessory.get_accessory()).into());

                        let accessories = PlatformAccessories::of(accessory.get_accessory());

                        homebridge.register_platform_accessories(
                            PLUGIN_IDENTIFIER,
                            PLATFORM_NAME,
                            accessories.clone()
                        );

                        devices.borrow_mut().push(accessory);
                    }
                }
            });
        }) as Box<dyn FnMut()>);

        let _ = set_interval(refresh_closure.as_ref().unchecked_ref(), REFRESH_RATE);
        refresh_closure.forget();
    }

    fn remove_cached(homebridge: &Homebridge, cached: Rc<RefCell<Vec<Accessory>>>) {
        for cached_accessory in cached.borrow().iter() {
            let accessories = PlatformAccessories::of(cached_accessory);

            homebridge.unregister_platform_accessories(
                PLUGIN_IDENTIFIER,
                PLATFORM_NAME,
                accessories
            );
        }

        cached.borrow_mut().clear();
    }

    #[wasm_bindgen(js_name = configureAccessory)]
    pub fn configure_accessory(&mut self, accessory: Accessory) {
        // Called by HomeBridge to restore cached accessories.
        self.cached.borrow_mut().push(accessory);
    }
}
