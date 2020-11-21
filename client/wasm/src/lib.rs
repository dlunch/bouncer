extern crate alloc;

mod app;

use wasm_bindgen::prelude::wasm_bindgen;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(module = "proto/BouncerServiceClientPb")]
extern "C" {
    type BouncerClient;

    #[wasm_bindgen(constructor)]
    fn new(hostname: &str) -> BouncerClient;
}

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    #[cfg(debug_assertions)]
    console_log::init_with_level(log::Level::Trace).unwrap();

    let _ = BouncerClient::new("test");

    yew::start_app::<app::App>();
}
