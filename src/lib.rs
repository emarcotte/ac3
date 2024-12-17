#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
// Disallow mod.rs, its too confusing to see a bunch of mod.rs files in various tools.
#![forbid(clippy::mod_module_files)]

pub mod ac3;
pub mod backtrack;
mod impls;
pub mod variable_provider;
