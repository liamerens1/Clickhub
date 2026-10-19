//! Mirrors `src/config/`.

// Integration-test crate: helpers outside #[test] fns may panic on setup failure.
#![allow(clippy::expect_used, clippy::unwrap_used)]

mod load;
mod parse;
mod shortcut;
