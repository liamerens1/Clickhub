import { emit, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { z } from "zod";

/**
 * Events between the Rust overlay controller (`src-tauri/src/overlay/`) and the overlay page.
 * Every payload from Rust is validated before use; a bad payload is reported and ignored.
 */

export const ringConfigSchema = z.object({
  slots: z.array(z.object({ label: z.string(), hint: z.string() })),
});

export const inkSchema = z.enum(["dark", "light"]);

export const ringOpenSchema = z.object({
  seq: z.number().int(),
  cx: z.number(),
  cy: z.number(),
  /** Radians clockwise from the top, one per slot (squeezed near screen edges). */
  angles: z.array(z.number()),
  /** Angle between neighbouring slots. */
  spacing: z.number().positive(),
  /** Stretches lines and labels outward (1 normally, more when squeezed near an edge). */
  radiusScale: z.number().positive(),
  /** Ink per slot, chosen by Rust against the screen behind each label. */
  inks: z.array(inkSchema),
  centreInk: inkSchema,
  emittedMs: z.number(),
});

export const ringAimSchema = z.object({
  seq: z.number().int(),
  /** Aim direction, radians clockwise from the top. */
  angle: z.number(),
  nearest: z.number().int().nonnegative().nullable(),
});

export const ringCloseSchema = z.object({
  seq: z.number().int(),
  fired: z.number().int().nonnegative().nullable(),
});

export type Ink = z.infer<typeof inkSchema>;
export type RingConfig = z.infer<typeof ringConfigSchema>;
export type RingOpen = z.infer<typeof ringOpenSchema>;
export type RingAim = z.infer<typeof ringAimSchema>;
export type RingClose = z.infer<typeof ringCloseSchema>;

/**
 * Listens to `name` events addressed to THIS overlay window only. Each monitor has its own
 * overlay and Rust addresses every ring event to one of them; the global `listen()` would hear
 * events for every overlay and draw a duplicate ring on the other monitor.
 */
function onEvent<T>(
  name: string,
  schema: z.ZodType<T>,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return getCurrentWebviewWindow().listen<unknown>(name, ({ payload }) => {
    const result = schema.safeParse(payload);
    if (result.success) {
      handler(result.data);
    } else {
      console.error(`ClickHub overlay: ignoring malformed ${name}`, result.error.issues);
    }
  });
}

export const onRingConfig = (handler: (config: RingConfig) => void) =>
  onEvent("ring:config", ringConfigSchema, handler);
export const onRingOpen = (handler: (open: RingOpen) => void) =>
  onEvent("ring:open", ringOpenSchema, handler);
export const onRingAim = (handler: (aim: RingAim) => void) =>
  onEvent("ring:aim", ringAimSchema, handler);
export const onRingClose = (handler: (close: RingClose) => void) =>
  onEvent("ring:close", ringCloseSchema, handler);

/** Tells Rust this overlay page is listening, so it sends the ring contents. */
export function emitOverlayReady(): Promise<void> {
  return emit("overlay:ready", { label: getCurrentWebviewWindow().label });
}

/** Reports the widest label at full zoom, so Rust can fit the ring near screen edges. */
export function emitOverlayExtent(maxLabelWidth: number): Promise<void> {
  return emit("overlay:extent", { label: getCurrentWebviewWindow().label, maxLabelWidth });
}

/** Reports the delay from Rust's `ring:open` to the first frame after the reveal. */
export function emitOverlayFrame(seq: number, emitToFrameMs: number): Promise<void> {
  return emit("overlay:frame", { seq, emitToFrameMs });
}
