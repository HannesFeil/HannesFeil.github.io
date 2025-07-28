//! The main application module

use website::App;

/// Entry point to the website
fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<App>::new().render();
}
