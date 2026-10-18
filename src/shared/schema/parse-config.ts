import type { z } from "zod";

import { type Config, configSchema } from "./config";

export type ParseConfigResult = { ok: true; config: Config } | { ok: false; error: string };

/** Parses raw config JSON. Never throws: every failure becomes `{ ok: false }`. */
export function parseConfig(json: string): ParseConfigResult {
  let raw: unknown;
  try {
    raw = JSON.parse(json);
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Config is not valid JSON: ${reason}` };
  }

  const result = configSchema.safeParse(raw);
  if (!result.success) {
    return { ok: false, error: formatIssues(result.error) };
  }
  return { ok: true, config: result.data };
}

function formatIssues(error: z.ZodError): string {
  return error.issues
    .map((issue) => {
      const path = issue.path.map(String).join(".");
      return `${path === "" ? "(root)" : path}: ${issue.message}`;
    })
    .join("; ");
}
