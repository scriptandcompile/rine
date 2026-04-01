//! Registry emulation — in-memory hierarchical key-value store.
//!
//! Backed by TOML hive files stored under `~/.rine/registry/`.
//! Each predefined root key (HKLM, HKCU, etc.) maps to a separate
//! TOML file.  Sub-keys are nested TOML tables, values are stored
//! as typed entries.
//!
//! The actual `extern "win64"` registry API entry-points live in
//! `rine64-advapi32::registry`.
