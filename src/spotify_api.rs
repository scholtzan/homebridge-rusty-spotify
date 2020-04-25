use wasm_bindgen::prelude::*;
use std::rc::Rc;
use js_sys::{Array, Function, Promise, Date};
use web_sys::{console, Request, RequestInit, RequestMode, Response};
use base64::encode;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::{spawn_local, future_to_promise};
use wasm_bindgen::JsCast;
use futures::Future;
use crate::node_fetch::{fetch, FetchMethod};
use std::collections::HashMap;

const ACCESS_TOKEN_LIFETIME: f64 = 3000.0;

#[derive(Serialize, Deserialize)]
struct SpotifyAuthorizationResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64
}

#[wasm_bindgen]
pub struct SpotifyApi {
    client_id: String,
    client_secret: String,
    access_token: Rc<String>,
    access_token_timestamp: Rc<f64>
}

#[wasm_bindgen]
impl SpotifyApi {
    #[wasm_bindgen(constructor)]
    pub fn new(client_id: String, client_secret: String) -> SpotifyApi {
        SpotifyApi {
            client_id,
            client_secret,
            access_token: Rc::new("".to_owned()),
            access_token_timestamp: Rc::new(0.0)
        }
    }

    pub fn play(&self) -> Promise {
        let authorize_request = self.authorize();

        future_to_promise(async move {
            console::log_1(&"Start playback".into());
            let access_token: String = JsFuture::from(authorize_request).await.unwrap().as_string().unwrap();

            let url = "https://api.spotify.com/v1/me/player/play";
            let authorization_header = format!("Bearer {}", access_token);

            let mut headers = HashMap::new();
            headers.insert("Authorization".to_owned(), authorization_header);

            let result = fetch(url, FetchMethod::Put, "", headers).await.unwrap();
            console::log_1(&format!("Playback result {:?}", result).into());
            Ok(JsValue::NULL)
        })
    }

    pub fn pause(&self) -> Promise {
        let authorize_request = self.authorize();

        future_to_promise(async move {
            console::log_1(&"Stop playback".into());
            let access_token: String = JsFuture::from(authorize_request).await.unwrap().as_string().unwrap();

            let url = "https://api.spotify.com/v1/me/player/pause";
            let authorization_header = format!("Bearer {}", access_token);

            let mut headers = HashMap::new();
            headers.insert("Authorization".to_owned(), authorization_header);

            let result = fetch(url, FetchMethod::Put, "", headers).await.unwrap();
            console::log_1(&format!("Playback stop result {:?}", result).into());
            Ok(JsValue::NULL)
        })
    }

    pub fn authorize(&self) -> Promise {
        console::log_1(&"Authorize to Spotify".into());

        let mut access_token = Rc::clone(&self.access_token);
        let mut access_token_timestamp = Rc::clone(&self.access_token_timestamp);

        let url = "https://accounts.spotify.com/api/token";
        let token = format!("{}:{}", self.client_id, self.client_secret);
        let base64_token = encode(token);
        let authorization_header = format!("Basic {}", base64_token);

        future_to_promise(async move {
            console::log_1(&"Get new access token from Spotify".into());

            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_owned(), "application/x-www-form-urlencoded;charset=UTF-8".to_owned());
            headers.insert("Authorization".to_owned(), authorization_header);

            let mut body = "grant_type=client_credentials";

            if Date::now() - *access_token_timestamp <= ACCESS_TOKEN_LIFETIME {
                console::log_1(&"Use cached access token".into());
                return Ok(JsValue::from((*access_token).clone()))
            }

            let result = fetch(url, FetchMethod::Post, body, headers).await.unwrap();

            let json: SpotifyAuthorizationResponse = result.into_serde().unwrap();
            access_token_timestamp = Rc::new(Date::now());
            access_token = Rc::new(json.access_token.clone());
            Ok(JsValue::from(json.access_token))
        })
    }
}

