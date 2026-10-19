//! Config schema, parsing, validation and loading. The only module that reads or writes the
//! config file. Never panics on bad input.

mod error;
mod load;
mod parse;
mod ring;
mod schema;
mod shortcut;
mod trigger;

pub use error::ConfigError;
pub use load::{CONFIG_FILE, Fallback, LoadStatus, Loaded, load};
pub use parse::parse;
pub use ring::{
    Action, LaunchAction, MAX_LABEL_CHARS, MAX_SLOTS, MIN_SLOTS, Ring, ShortcutAction, Slot,
    UrlAction,
};
pub use schema::{CONFIG_VERSION, Config};
pub use shortcut::{Key, NamedKey, Shortcut};
pub use trigger::{KeyboardTrigger, Modifier, MouseButton, MouseTrigger, Trigger};
