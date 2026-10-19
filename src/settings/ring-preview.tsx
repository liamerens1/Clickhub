import type { Slot } from "../shared/schema";

interface RingPreviewProps {
  slots: readonly Slot[];
  /** The slot to emphasise (hovered or being edited), if any. */
  focus: number | null;
}

const INNER = 30;
const OUTER = 74;
const LABEL = 84;
const MAX_PREVIEW_CHARS = 14;

const short = (label: string) =>
  Array.from(label).length > MAX_PREVIEW_CHARS
    ? `${Array.from(label)
        .slice(0, MAX_PREVIEW_CHARS - 1)
        .join("")}…`
    : label;

/**
 * A small drawing of the ring as it will open: plain lines from an empty centre to plain
 * labels, clockwise from the top. Not interactive; it follows the list.
 */
export function RingPreview({ slots, focus }: RingPreviewProps) {
  const count = slots.length;
  const centre = focus !== null ? slots[focus]?.label : undefined;
  return (
    <svg className="preview-svg" viewBox="-170 -122 340 244" role="img" aria-label="Ring preview">
      {slots.map((slot, index) => {
        const angle = (index / count) * Math.PI * 2;
        const sin = Math.sin(angle);
        const cos = -Math.cos(angle);
        const anchor = Math.abs(sin) < 0.08 ? "middle" : sin > 0 ? "start" : "end";
        const labelY = cos * LABEL + (Math.abs(sin) < 0.08 ? cos * 5 : 0);
        return (
          <g
            // The preview is redrawn from the list; positions are the identity here.
            key={index}
            className={focus === index ? "preview-spoke is-focus" : "preview-spoke"}
          >
            <line
              className="preview-line"
              x1={sin * INNER}
              y1={cos * INNER}
              x2={sin * OUTER}
              y2={cos * OUTER}
            />
            <text
              className="preview-label"
              x={sin * LABEL}
              y={labelY}
              textAnchor={anchor}
              dominantBaseline="middle"
            >
              {short(slot.label)}
            </text>
          </g>
        );
      })}
      <text className="preview-centre" x={0} y={0} textAnchor="middle" dominantBaseline="middle">
        {centre !== undefined ? short(centre) : `${String(count)} slots`}
      </text>
    </svg>
  );
}
