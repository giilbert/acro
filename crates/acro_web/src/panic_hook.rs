use std::panic::PanicInfo;
use wasm_bindgen::throw_str;

pub fn panic_hook(info: &PanicInfo) {
    throw_str(&format!("WebAssembly module panicked!\n\n{info}"));
}
