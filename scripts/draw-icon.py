"""Draws the ClickHub app icon (the sidebar's spoke ring) as a 1024 px PNG.

Usage: python scripts/draw-icon.py <out.png>
Then: pnpm tauri icon <out.png>  (generates every size in src-tauri/icons)

The glyph matches `ring` in src/settings/icon.tsx: eight spokes on a 16-unit grid, from radius
3.5 to 6.5 around the centre, round caps, drawn in the dark-theme accent on a dark rounded square
so it reads on light and dark taskbars alike. Drawn 4x larger and downsampled for smooth edges.
"""

import math
import sys

from PIL import Image, ImageDraw

SIZE = 1024
SCALE = 4
CANVAS = SIZE * SCALE
BACKGROUND = (32, 32, 32, 255)
BORDER = (255, 255, 255, 22)
ACCENT = (76, 194, 255, 255)
CORNER = 0.225  # corner radius as a share of the side, like Windows 11 app tiles
INNER, OUTER = 3.5, 6.5  # spoke ends, in glyph units
STROKE = 1.55  # glyph units; a little heavier than the 1.3 UI stroke so 16 px stays legible
GLYPH_SHARE = 0.66  # the glyph's outer diameter as a share of the side


def main() -> None:
    out = sys.argv[1]
    image = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    radius = int(CANVAS * CORNER)
    draw.rounded_rectangle((0, 0, CANVAS - 1, CANVAS - 1), radius, fill=BACKGROUND)
    border = max(1, CANVAS // 256)
    draw.rounded_rectangle(
        (border // 2, border // 2, CANVAS - 1 - border // 2, CANVAS - 1 - border // 2),
        radius,
        outline=BORDER,
        width=border,
    )

    unit = CANVAS * GLYPH_SHARE / (2 * OUTER + STROKE)
    centre = CANVAS / 2
    width = STROKE * unit
    for spoke in range(8):
        angle = spoke * math.pi / 4
        dx, dy = math.sin(angle), -math.cos(angle)
        start = (centre + dx * INNER * unit, centre + dy * INNER * unit)
        end = (centre + dx * OUTER * unit, centre + dy * OUTER * unit)
        draw.line((start, end), fill=ACCENT, width=round(width))
        for x, y in (start, end):
            r = width / 2
            draw.ellipse((x - r, y - r, x + r, y + r), fill=ACCENT)

    image.resize((SIZE, SIZE), Image.Resampling.LANCZOS).save(out)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
