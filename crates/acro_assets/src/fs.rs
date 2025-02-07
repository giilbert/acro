#[cfg(target_arch = "wasm32")]
use crate::wasm;

pub fn read(path: impl AsRef<str>) -> eyre::Result<Vec<u8>> {
    let path = path.as_ref();

    // TODO: not copy memory so many times
    #[cfg(target_arch = "wasm32")]
    return Ok(Vec::from_iter(wasm::get_asset(path)?.iter().cloned()));

    #[cfg(not(target_arch = "wasm32"))]
    return Ok(std::fs::read(&path)?);
}

pub fn read_to_string(path: impl AsRef<str>) -> eyre::Result<String> {
    Ok(String::from_utf8(read(path)?)?)
}
