import { z } from "zod";

/**
 * What opens the ring. Mirrors `src-tauri/src/config/trigger.rs`; the shared fixtures in
 * `fixtures/config` keep the two in agreement.
 */

/** `left` is deliberately absent: a bad config must never take over the primary button. */
export const mouseButtonSchema = z.enum(["right", "middle", "x1", "x2"]);

export const modifierSchema = z.enum(["ctrl", "shift", "alt", "win"]);

export const mouseTriggerSchema = z
  .object({
    kind: z.literal("mouse"),
    button: mouseButtonSchema,
    /** Exact set of modifiers that must be held. Empty means "no modifiers held". */
    modifiers: z
      .array(modifierSchema)
      .refine((list) => new Set(list).size === list.length, {
        message: "a modifier is listed more than once",
      })
      .default([]),
    /** Hide the trigger click from the app underneath. On by default since Phase 2. */
    swallow: z.boolean().default(true),
  })
  .strict();

export const keyboardTriggerSchema = z
  .object({
    kind: z.literal("keyboard"),
    /** Registered through tauri-plugin-global-shortcut, e.g. "Ctrl+Alt+Space". */
    shortcut: z.string().trim().min(1, "keyboard trigger shortcut must not be empty"),
  })
  .strict();

export const triggerSchema = z.discriminatedUnion("kind", [
  mouseTriggerSchema,
  keyboardTriggerSchema,
]);

export const DEFAULT_TRIGGER: Trigger = {
  kind: "mouse",
  button: "x2",
  modifiers: [],
  swallow: true,
};

export type MouseButton = z.infer<typeof mouseButtonSchema>;
export type Modifier = z.infer<typeof modifierSchema>;
export type MouseTrigger = z.infer<typeof mouseTriggerSchema>;
export type KeyboardTrigger = z.infer<typeof keyboardTriggerSchema>;
export type Trigger = z.infer<typeof triggerSchema>;
