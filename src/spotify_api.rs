//! Represent the Spotify API.

use crate::node_fetch::{fetch, FetchMethod};
use base64::encode;
use js_sys::{Date, Promise};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;

/// The Spotify API access token needs to be refreshed after 50 minutes.
const ACCESS_TOKEN_LIFETIME: f64 = 3000.0;
/// Path to the Homebridge config file.
const HOMEBRIDGE_CONFIG: &str = "~/.homebridge/config.json";

#[wasm_bindgen]
extern "C" {
    type Fs;

    fn require(name: &str) -> Fs;

    #[wasm_bindgen(method, js_name = readFileSync)]
    fn read_file(this: &Fs, file: &str) -> String;

    #[wasm_bindgen(method, js_name = writeFileSync)]
    fn write_file(this: &Fs, file: &str, data: String) -> String;
}

#[derive(Serialize, Deserialize)]
/// Represents the response when making an authorization request.
struct SpotifyAuthorizationResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
}

#[derive(Serialize, Deserialize)]
/// Represents information of a device returned by the Spotify Web API.
struct SpotifyDevice {
    pub id: String,
    pub is_active: bool,
    pub volume_percent: u32,
    // ... more attributes ...
}

#[derive(Serialize, Deserialize)]
/// Represents the response when requesting the playback state.
struct SpotifyPlaybackResponse {
    pub device: SpotifyDevice,
    pub is_playing: bool,
    // ... more attributes ...
}

#[wasm_bindgen]
/// Represents the Spotify API and state.
pub struct SpotifyApi {
    client_id: String,
    client_secret: String,
    access_token: Rc<String>,
    refresh_token: Rc<String>,
    access_token_timestamp: Rc<f64>,
}

#[wasm_bindgen]
impl SpotifyApi {
    #[wasm_bindgen(constructor)]
    pub fn new(client_id: String, client_secret: String, refresh_token: String) -> SpotifyApi {
        SpotifyApi {
            client_id,
            client_secret,
            access_token: Rc::new("".to_owned()),
            refresh_token: Rc::new(refresh_token),
            access_token_timestamp: Rc::new(0.0),
        }
    }

    /// Make a request to start playing music.
    pub fn play(&self, device_id: Option<String>) -> Promise {
        let authorize_request = self.authorize();

        future_to_promise(async move {
            console::log_1(&"Start playback".into());

            match JsFuture::from(authorize_request).await {
                Ok(authorize_request) => {
                    let access_token: String = authorize_request.as_string().unwrap();

                    let mut url = "https://api.spotify.com/v1/me/player/play".to_owned();

                    if let Some(device_id) = device_id {
                        url.push_str(&format!("?device_id={}", device_id));
                    }

                    let authorization_header = format!("Bearer {}", access_token);

                    let mut headers = HashMap::new();
                    headers.insert("Authorization".to_owned(), authorization_header);

                    match fetch(&url, FetchMethod::Put, "", headers, true).await {
                        Err(e) => {
                            console::log_1(&format!("Error starting playback: {:?}", e).into())
                        }
                        Ok(_) => {} // player successfully started
                    }
                }
                Err(e) => console::log_1(
                    &format!("Error while authenticating to Spotify API: {:?}", e).into(),
                ),
            }

            Ok(JsValue::NULL)
        })
    }

    /// Make a request to pause Spotify.
    pub fn pause(&self) -> Promise {
        let authorize_request = self.authorize();

        future_to_promise(async move {
            console::log_1(&"Stop playback".into());
            match JsFuture::from(authorize_request).await {
                Ok(authorize_request) => {
                    let access_token: String = authorize_request.as_string().unwrap();

                    let url = "https://api.spotify.com/v1/me/player/pause";
                    let authorization_header = format!("Bearer {}", access_token);

                    let mut headers = HashMap::new();
                    headers.insert("Authorization".to_owned(), authorization_header);

                    match fetch(url, FetchMethod::Put, "", headers, true).await {
                        Err(e) => {
                            console::log_1(&format!("Error stopping playback: {:?}", e).into())
                        }
                        Ok(_) => {} // player successfully stopped playing
                    }
                }
                Err(e) => console::log_1(
                    &format!("Error while authenticating to Spotify API: {:?}", e).into(),
                ),
            }

            Ok(JsValue::NULL)
        })
    }

    /// Check if Spotify is currently playing; optionally check for a specific device
    pub fn is_playing(&self, device_id: Option<String>) -> Promise {
        let authorize_request = self.authorize();

        future_to_promise(async move {
            console::log_1(&"Check playback state".into());

            match JsFuture::from(authorize_request).await {
                Ok(authorize_request) => {
                    let access_token: String = authorize_request.as_string().unwrap();

                    let url = "https://api.spotify.com/v1/me/player";
                    let authorization_header = format!("Bearer {}", access_token);

                    let mut headers = HashMap::new();
                    headers.insert("Authorization".to_owned(), authorization_header);

                    match fetch(url, FetchMethod::Get, "", headers, false).await {
                        Err(e) => {
                            console::log_1(&format!("Error getting playback state: {:?}", e).into())
                        }
                        Ok(result) => {
                            let json: SpotifyPlaybackResponse = result.into_serde().unwrap();

                            if device_id.is_some() && device_id.unwrap() == json.device.id {
                                return Ok(JsValue::from(json.is_playing));
                            } else {
                                return Ok(JsValue::from(json.is_playing));
                            }
                        }
                    }
                }
                Err(e) => console::log_1(
                    &format!("Error while authenticating to Spotify API: {:?}", e).into(),
                ),
            }

            Ok(JsValue::FALSE)
        })
    }

    /// Check if Spotify is currently playing; optionally check for a specific device
    pub fn get_volume(&self, device_id: Option<String>) -> Promise {
        let authorize_request = self.authorize();

        future_to_promise(async move {
            console::log_1(&"Check playback state".into());

            match JsFuture::from(authorize_request).await {
                Ok(authorize_request) => {
                    let access_token: String = authorize_request.as_string().unwrap();

                    let url = "https://api.spotify.com/v1/me/player";
                    let authorization_header = format!("Bearer {}", access_token);

                    let mut headers = HashMap::new();
                    headers.insert("Authorization".to_owned(), authorization_header);

                    match fetch(url, FetchMethod::Get, "", headers, false).await {
                        Err(e) => {
                            console::log_1(&format!("Error getting volume state: {:?}", e).into())
                        }
                        Ok(result) => {
                            let json: SpotifyPlaybackResponse = result.into_serde().unwrap();

                            if device_id.is_some() && device_id.unwrap() == json.device.id {
                                return Ok(JsValue::from(json.device.volume_percent));
                            } else {
                                return Ok(JsValue::from(json.device.volume_percent));
                            }
                        }
                    }
                }
                Err(e) => console::log_1(
                    &format!("Error while authenticating to Spotify API: {:?}", e).into(),
                ),
            }

            Ok(JsValue::from(100))
        })
    }

    /// Make a request to update the volume on the active device.
    pub fn set_volume(&self, volume: u32) -> Promise {
        let authorize_request = self.authorize();

        future_to_promise(async move {
            console::log_1(&"Set volume".into());
            match JsFuture::from(authorize_request).await {
                Ok(authorize_request) => {
                    let access_token: String = authorize_request.as_string().unwrap();

                    let url = format!(
                        "https://api.spotify.com/v1/me/player/volume?volume_percent={}",
                        volume
                    );
                    let authorization_header = format!("Bearer {}", access_token);

                    let mut headers = HashMap::new();
                    headers.insert("Authorization".to_owned(), authorization_header);

                    match fetch(&url, FetchMethod::Put, "", headers, true).await {
                        Err(e) => console::log_1(&format!("Error changing volume: {:?}", e).into()),
                        Ok(_) => {} // volume successfully updated
                    }
                }
                Err(e) => console::log_1(
                    &format!("Error while authenticating to Spotify API: {:?}", e).into(),
                ),
            }

            Ok(JsValue::NULL)
        })
    }

    /// Make an authorization request.
    pub fn authorize(&self) -> Promise {
        console::log_1(&"Authorize to Spotify".into());

        let mut refresh_token = Rc::clone(&self.refresh_token);
        let mut access_token = Rc::clone(&self.access_token);
        let mut access_token_timestamp = Rc::clone(&self.access_token_timestamp);

        let url = "https://accounts.spotify.com/api/token";
        let token = format!("{}:{}", self.client_id, self.client_secret);
        let base64_token = encode(token);
        let authorization_header = format!("Basic {}", base64_token);

        future_to_promise(async move {
            let mut headers = HashMap::new();
            headers.insert(
                "Content-Type".to_owned(),
                "application/x-www-form-urlencoded;charset=UTF-8".to_owned(),
            );
            headers.insert("Authorization".to_owned(), authorization_header);

            let body = format!("grant_type=refresh_token&refresh_token={}", *refresh_token);

            if Date::now() - *access_token_timestamp <= ACCESS_TOKEN_LIFETIME {
                return Ok(JsValue::from((*access_token).clone()));
            }

            if let Ok(result) = fetch(url, FetchMethod::Post, &body, headers, false).await {
                let json: Result<SpotifyAuthorizationResponse, _> = result.into_serde();

                match json {
                    Ok(json) => {
                        access_token_timestamp = Rc::new(Date::now());
                        access_token = Rc::new(json.access_token.clone());

                        if let Some(new_refresh_token) = json.refresh_token {
                            // cache refresh token
                            let fs = require("fs");

                            let config_string = fs.read_file(HOMEBRIDGE_CONFIG);
                            let new_config_string = config_string
                                .replace(&(*refresh_token), &new_refresh_token.clone());
                            fs.write_file(HOMEBRIDGE_CONFIG, new_config_string);
                            refresh_token = Rc::new(new_refresh_token);
                        }

                        return Ok(JsValue::from(json.access_token));
                    }
                    Err(_) => {
                        console::log_1(
                            &format!("Error while retrieving access token from Spotify API. Response was: {:?}", result).into(),
                        );
                        return Err(JsValue::from(format!("Error while retrieving access token from Spotify API. Response was: {:?}", result)));
                    }
                }
            } else {
                Err(JsValue::from("Error executing fetch request"))
            }
        })
    }
}
