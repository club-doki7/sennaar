use lalrpop_util::lalrpop_mod;

mod platform;
mod metadata;
mod rawtype;
mod entity;
mod registry;

lalrpop_mod!(lalr, "/registry/lalr.rs");

pub use metadata::*;
pub use platform::*;
pub use rawtype::*;
pub use entity::*;
pub use registry::*;