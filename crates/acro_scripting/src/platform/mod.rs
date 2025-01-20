#[cfg(not(target_arch = "wasm32"))]
pub mod deno_ops;
mod function;

pub use function::FunctionHandle;
