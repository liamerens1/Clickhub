import { z } from "zod";

import { shortcutSchema } from "./shortcut";

/** Mirrors `src-tauri/src/config/ring.rs`; shared fixtures keep the two in agreement. */

export const MIN_SLOTS = 2;
export const MAX_SLOTS = 12;
export const MAX_LABEL_CHARS = 32;

const labelSchema = z
  .string()
  .trim()
  .min(1, "slot label must not be empty")
  .refine((label) => Array.from(label).length <= MAX_LABEL_CHARS, {
    message: `slot label must be at most ${String(MAX_LABEL_CHARS)} characters`,
  });

export const actionSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("shortcut"), keys: shortcutSchema }).strict(),
  z
    .object({
      kind: z.literal("launch"),
      path: z.string().trim().min(1, "launch path must not be empty"),
      args: z.array(z.string()).default([]),
    })
    .strict(),
  z
    .object({
      kind: z.literal("url"),
      url: z
        .string()
        .regex(
          /^https?:\/\/\S+$/i,
          "URL must start with http:// or https:// and contain no spaces",
        ),
    })
    .strict(),
]);

export const slotSchema = z.object({ label: labelSchema, action: actionSchema }).strict();

export const ringSchema = z
  .object({ slots: z.array(slotSchema).min(MIN_SLOTS).max(MAX_SLOTS) })
  .strict();

export type Action = z.infer<typeof actionSchema>;
export type Slot = z.infer<typeof slotSchema>;
export type Ring = z.infer<typeof ringSchema>;

export const DEFAULT_RING: Ring = {
  slots: [
    { label: "Copy", action: { kind: "shortcut", keys: "Ctrl+C" } },
    { label: "Paste", action: { kind: "shortcut", keys: "Ctrl+V" } },
    { label: "Screenshot", action: { kind: "shortcut", keys: "Win+Shift+S" } },
    { label: "File Explorer", action: { kind: "launch", path: "explorer.exe", args: [] } },
    { label: "Browser", action: { kind: "url", url: "https://www.google.com" } },
    { label: "Show desktop", action: { kind: "shortcut", keys: "Win+D" } },
  ],
};
