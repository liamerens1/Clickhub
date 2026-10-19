//! Config schema, parsing, validation and loading. The only module that reads or writes the
//! config file. Never panics on bad input.

mod error;
mod load;
mod parse;
mod schema;
mod trigger;

pub use error::ConfigError;
pub use load::{CONFIG_FILE, Fallback, LoadStatus, Loaded, load};
pub use parse::parse;
pub use schema::{CONFIG_VERSION, Config};
pub use trigger::{KeyboardTrigger, Modifier, MouseButton, MouseTrigger, Trigger};
