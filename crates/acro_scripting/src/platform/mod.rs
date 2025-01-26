#[cfg(not(target_arch = "wasm32"))]
pub mod deno_ops;
#[cfg(target_arch = "wasm32")]
pub mod wasm_ops;

mod function;
pub mod ops;

pub use function::FunctionHandle;
