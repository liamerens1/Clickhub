import { describe, expect, it } from "vitest";

import { slotSchema } from "../../src/shared/schema";

import { PRESET_GROUPS } from "../../src/settings/presets";

describe("presets", () => {
  it("are all valid slots", () => {
    for (const group of PRESET_GROUPS) {
      for (const slot of group.slots) {
        const result = slotSchema.safeParse(slot);
        expect(result.success, `${group.name}: ${slot.label}`).toBe(true);
        // Already canonical, so picking one never changes what is saved.
        expect(result.data).toEqual(slot);
      }
    }
  });

  it("have unique labels within a group", () => {
    for (const group of PRESET_GROUPS) {
      const labels = group.slots.map((slot) => slot.label);
      expect(new Set(labels).size).toBe(labels.length);
    }
  });
});
