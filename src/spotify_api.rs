use wasm_bindgen::prelude::*;
use std::rc::Rc;
use web_sys::{console, Request, RequestInit, RequestMode, Response};
use base64::encode;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use futures::executor::block_on;

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

        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);

        let url = "https://accounts.spotify.com/api/token";

        match Request::new_with_str_and_init(url, &opts) {
            Ok(request) => {
                let token = format!("{}:{}", client_id, client_secret);
                let base64_token = encode(token);
                let authorization_header = format!("Basic {}", base64_token);

                console::log_1(&format!("base64 token {:?}", base64_token).into());

                request.headers().set("Authorization", &authorization_header);

                let window = web_sys::window().unwrap();

                block_on(async {
                    match JsFuture::from(window.fetch_with_request(&request)).await {
                        Ok(response) => {
                            let resp: Response = response.dyn_into().unwrap();

                            match resp.json() {
                                Ok(json_future) => {
                                    match JsFuture::from(json_future).await {
                                        Ok(json) => {
                                            let authorization_response: SpotifyAuthorizationResponse = json.into_serde().unwrap();
                                            access_token = Rc::new(authorization_response.access_token);

                                            console::log_1(&"Access token".into());
                                            console::log_1(&(&*access_token).into());
                                        },
                                        _ => console::log_1(&"Error processing Spotify authorization JSON".into())
                                    }
                                },
                                _ => console::log_1(&"Error JSON Spotify authorization response".into())
                            }
                        },
                        _ => console::log_1(&"Error processing Spotify authorization response".into())
                    }
                });
            },
            e => console::log_1(&format!("Error authorizing to Spotify {:?}", e).into())
        }
    }
}

