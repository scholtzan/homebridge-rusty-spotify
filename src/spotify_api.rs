use wasm_bindgen::prelude::*;
use std::rc::Rc;
use web_sys::{console, Request, RequestInit, RequestMode, Response};
use base64::encode;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use crate::node_fetch::{fetch, FetchMethod};
use std::collections::HashMap;


#[derive(Serialize, Deserialize)]
struct SpotifyAuthorizationResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64
}

#[wasm_bindgen]
pub struct SpotifyApi {
    client_id: Rc<String>,
    client_secret: Rc<String>,
    access_token: Rc<String>
}

#[wasm_bindgen]
impl SpotifyApi {
    #[wasm_bindgen(constructor)]
    pub fn new(client_id: String, client_secret: String) -> SpotifyApi {
        SpotifyApi {
            client_id: Rc::new(client_id),
            client_secret: Rc::new(client_secret),
            access_token: Rc::new("".to_owned()) // todo
        }
    }

    pub fn authorize(&self) {
        console::log_1(&"Authorize to Spotify".into());

        let client_id = Rc::clone(&self.client_id);
        let client_secret = Rc::clone(&self.client_secret);
        let mut access_token = Rc::clone(&self.access_token);

        let url = "https://accounts.spotify.com/api/token";
        let token = format!("{}:{}", client_id, client_secret);
        let base64_token = encode(token);
        let authorization_header = format!("Basic {}", base64_token);

        console::log_1(&format!("base64 token {:?}", base64_token).into());

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_owned(), "application/x-www-form-urlencoded;charset=UTF-8".to_owned());
        headers.insert("Authorization".to_owned(), authorization_header);

        let mut body = "grant_type=client_credentials";

        let r = spawn_local(async move {
            console::log_1(&format!("1").into());
            let result = fetch(url, FetchMethod::Post, body, headers).await.unwrap();
            console::log_1(&format!("2").into());

            let json: SpotifyAuthorizationResponse = result.into_serde().unwrap();
            console::log_1(&format!("3 {:?}", json.access_token).into());

        });

    }
}

