use std::{
    collections::HashMap,
    io::{Cursor, Read},
    sync::Arc,
};

use eyre::OptionExt;
use wasm_bindgen::prelude::*;
use zip::ZipArchive;

pub static mut ASSETS_FS: Option<HashMap<String, Arc<[u8]>>> = None;

#[wasm_bindgen(js_name = initAssetsBytes)]
pub fn init_assets_bytes(bytes: Box<[u8]>) -> Result<(), JsValue> {
    init_assets_bytes_inner(bytes).map_err(|e| JsValue::from_str(&format!("{:?}", e)))
}

fn init_assets_bytes_inner(bytes: Box<[u8]>) -> eyre::Result<()> {
    let bytes = Cursor::new(bytes);
    let mut archive = ZipArchive::new(bytes)?;

    let mut files: HashMap<String, Arc<[u8]>> = HashMap::new();

    for file_name in archive
        .file_names()
        .map(|f| String::from(f))
        .collect::<Vec<_>>()
    {
        let file = archive.by_name(&file_name)?;
        let buf = file.bytes().collect::<Result<Arc<[u8]>, _>>()?;
        files.insert(file_name, buf);
    }

    // SAFETY: This is safe because the single-threaded nature of WebAssembly guarantees Rust
    // ownership rules.
    unsafe {
        ASSETS_FS = Some(files);
    }

    Ok(())
}

pub fn get_asset(name: &str) -> eyre::Result<Arc<[u8]>> {
    // SAFETY: This is safe because the single-threaded nature of WebAssembly guarantees Rust
    // ownership rules.
    unsafe {
        Ok(ASSETS_FS
            .as_ref()
            .ok_or_eyre("asset filesystem not initialized")?
            .get(name)
            .ok_or_eyre("file not found")?
            .clone())
    }
}
