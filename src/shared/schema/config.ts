import { z } from "zod";

/**
 * Config file schema. Mirrors `src-tauri/src/config/schema.rs`; the shared
 * fixtures in `fixtures/config` keep the two in agreement.
 */
export const CONFIG_VERSION = 1;

export const configSchema = z
  .object({
    version: z.literal(CONFIG_VERSION),
  })
  .strict();

export type Config = z.infer<typeof configSchema>;
