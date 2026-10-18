//! Config schema and parsing. The only module that reads or writes the config
//! file (from Phase 2). Never panics on bad input.

mod error;
mod parse;
mod schema;

pub use error::ConfigError;
pub use parse::parse;
pub use schema::{CONFIG_VERSION, Config};
