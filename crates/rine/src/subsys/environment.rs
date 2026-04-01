//! Environment variable translation — re-exports the shared environment
//! store from `rine_types::environment` and provides any loader-specific
//! helpers.
//!
//! The core store lives in `rine-types` so that DLL implementation crates
//! (e.g. `rine64-kernel32`) can access it without depending on the `rine`
//! crate.

#[allow(unused_imports)]
pub use rine_types::environment::*;
