# Shared fixtures

Every file under `config/valid` must be accepted, and every file under `config/invalid`
rejected, by **both** config parsers:

- TypeScript (Zod): `tests/shared/schema/config-fixtures.test.ts`
- Rust (serde): `src-tauri/tests/config.rs`

If the two schemas drift apart, one of those suites fails. Add a fixture whenever you change
the config shape. Name files in kebab-case after what they test.
