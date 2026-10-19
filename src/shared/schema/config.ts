import { z } from "zod";

import { DEFAULT_TRIGGER, triggerSchema } from "./trigger";

/**
 * Config file schema. Mirrors `src-tauri/src/config/schema.rs`; the shared
 * fixtures in `fixtures/config` keep the two in agreement.
 */
export const CONFIG_VERSION = 1;

export const configSchema = z
  .object({
    version: z.literal(CONFIG_VERSION),
    trigger: triggerSchema.default(DEFAULT_TRIGGER),
  })
  .strict();

export type Config = z.infer<typeof configSchema>;
