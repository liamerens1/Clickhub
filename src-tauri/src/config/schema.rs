use serde::{Deserialize, Serialize};

/// Config file version this build understands. Mirrors `CONFIG_VERSION` in
/// `src/shared/schema/config.ts`.
pub const CONFIG_VERSION: u32 = 1;

/// Root of `config.json`. Mirrors `configSchema` in `src/shared/schema/config.ts`;
/// the shared fixtures in `fixtures/config` keep the two in agreement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub version: u32,
}
