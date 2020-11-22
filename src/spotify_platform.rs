//! Defines the Homebridge Spotify Platform.

use crate::spotify_api::SpotifyApi;
use js_sys::{Array, Function, Object};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;

use crate::spotify_accessory::SpotifyAccessory;
use crate::spotify_accessory::Accessory;

const REFRESH_RATE: u32 = 10 * 1000; // milliseconds
const PLUGIN_IDENTIFIER: &str = "homebridge-rusty-spotify";
const PLATFORM_NAME: &str = "Spotify";


#[wasm_bindgen]
extern "C" {
    #[derive(Clone)]
    pub type Service;

    #[derive(Clone)]
    pub type Homebridge;

    #[wasm_bindgen(js_name = Array)]
    pub type PlatformAccessories;

    #[wasm_bindgen(constructor, js_class = "Array")]
    fn of(accessory: &Accessory) -> PlatformAccessories;

    #[wasm_bindgen(method, js_name = registerPlatformAccessories)]
    fn register_platform_accessories(this: &Homebridge, plugin_identifier: &str, platform_name: &str, accessories: PlatformAccessories);

    // #[wasm_bindgen(method, js_name = unregisterPlatformAccessories)]
    // fn unregister_platform_accessories(this: &Homebridge, plugin_identifier: &str, platform_name: &str, accessories: Array);

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
    /// Reference to the Service object
    service: Service,
    /// API to control Spotify
    api: Rc<SpotifyApi>,
    /// Available Spotify devices
    devices: Vec<SpotifyAccessory>,

    #[wasm_bindgen(js_name = pollingInterval)]
    pub polling_interval: i64
}

#[wasm_bindgen]
impl SpotifyPlatform {
    #[wasm_bindgen(constructor)]
    pub fn new(homebridge: Homebridge, service: Service, _log: Function, config: &JsValue) -> SpotifyPlatform {
        let config: Config = config.into_serde().unwrap();

        let api = SpotifyApi::new(
            config.client_id.clone(),
            config.client_secret.clone(),
            config.refresh_token.clone(),
        );

        let platform = SpotifyPlatform {
            homebridge,
            config,
            service,
            api: Rc::new(api),
            devices: Vec::new(),
            polling_interval: 1000
        };

        platform.refresh_devices();
        platform
    }

    /// Determine available Spotify devices and add them as accessories,
    /// remove devices that became inactive.
    fn refresh_devices(&self) {
        let homebridge = self.homebridge.clone();
        let service = self.service.clone();
        let api = self.api.clone();

        let refresh_closure = Closure::wrap(Box::new(move || {
            let homebridge = homebridge.clone();
            let service = service.clone();
            let api = api.clone();

            console::log_1(&"Check device statuses".into());

            spawn_local(async move {
                console::log_1(&"Do check".into());
                let device_id = "test".to_string();
                let name = "test".to_string();

                let accessory = SpotifyAccessory::new(
                    service,
                    name,
                    device_id,
                    api);

                let accessories = PlatformAccessories::of(accessory.get_accessory());

                homebridge.register_platform_accessories(
                    PLUGIN_IDENTIFIER,
                    PLATFORM_NAME,
                    accessories
                )
            });
        }) as Box<dyn FnMut()>);

        let _ = set_interval(refresh_closure.as_ref().unchecked_ref(), REFRESH_RATE);

        refresh_closure.forget();
        // api request
        // update self.devices

        // console::log_1(&format!("Adding new device: {:?}", device).into());

    }

    #[wasm_bindgen(js_name = configureAccessory)]
    pub fn configure_accessory(&self) {
        // Called by HomeBridge to restore cached accessories.
        // Ignored here since all accessories get created dynamically based
        // on available devices.
    }

}
