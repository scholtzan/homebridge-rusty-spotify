use wasm_bindgen::prelude::*;
use web_sys::console;
use js_sys::Function;
use js_sys::Array;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;


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
}

#[wasm_bindgen]
pub struct SpotifyAccessory {
    log: Function,
    config: JsValue,
    on: Rc<bool>,
    service_switch: Switch,
}

#[wasm_bindgen]
impl SpotifyAccessory {
    #[wasm_bindgen(constructor)]
    pub fn new(service_switch: Switch, log: Function, config: JsValue) -> SpotifyAccessory {
        console::log_1(&"Hello using web-sys".into());


        // config: https://rustwasm.github.io/docs/wasm-bindgen/reference/accessing-properties-of-untyped-js-values.html

        SpotifyAccessory {
            log,
            config,
            on: Rc::new(false),
            service_switch
        }
    }

    #[wasm_bindgen(js_name = getServices)]
    pub fn get_services(&mut self) -> Array {
        console::log_1(&"get Spotify service 1".into());

        let get_on = {
            console::log_1(&"get on".into());
            let on = Rc::clone(&self.on);
            Closure::wrap(Box::new(move |callback: Function| {

                console::log_1(&"get on".into());
                //callback.apply(&JsValue::null(), &Array::of2(&JsValue::null(), &JsValue::from(*on))).unwrap();
            }) as Box<dyn FnMut(Function)>)
        };


        let set_on = {
            let mut on = Rc::clone(&self.on);
            Closure::wrap(Box::new(move |new_on: bool, callback: Function| {
                on = Rc::new(new_on);
                callback.apply(&JsValue::null(), &Array::of2(&JsValue::null(), &JsValue::from(*on))).unwrap();
            }) as Box<dyn FnMut(bool, Function)>)
        };


        // https://stackoverflow.com/questions/53214434/how-to-return-a-rust-closure-to-javascript-via-webassembly
        let c = self.service_switch.get_characteristic("On");
        c.on("set", set_on.as_ref().unchecked_ref()).on("get", get_on.as_ref().unchecked_ref());

        get_on.forget();
        set_on.forget();

        console::log_1(&"get Spotify service".into());

        [self.service_switch.clone()].iter().collect()
    }
}