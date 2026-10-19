/** Small line icons drawn on a 16-unit grid with the current text colour. */
const PATHS = {
  ring: "M8 1.5v3M8 11.5v3M1.5 8h3M11.5 8h3M3.4 3.4l2.1 2.1M10.5 10.5l2.1 2.1M3.4 12.6l2.1-2.1M10.5 5.5l2.1-2.1",
  mouse:
    "M8 1.5a4.5 4.5 0 0 0-4.5 4.5v4a4.5 4.5 0 0 0 9 0V6A4.5 4.5 0 0 0 8 1.5zM8 1.5v5M3.5 6.5h9",
  gear: "M8 5.5a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5zM8 1.5v1.6M8 12.9v1.6M1.5 8h1.6M12.9 8h1.6M3.4 3.4l1.1 1.1M11.5 11.5l1.1 1.1M3.4 12.6l1.1-1.1M11.5 4.5l1.1-1.1",
  apps: "M2.5 2.5h4v4h-4zM9.5 2.5h4v4h-4zM2.5 9.5h4v4h-4zM9.5 9.5h4v4h-4z",
  palette:
    "M8 1.5a6.5 6.5 0 0 0 0 13c1 0 1.5-.6 1.5-1.4 0-1.1-.9-1.4-.9-2.3 0-.8.6-1.3 1.4-1.3h1.6a2.9 2.9 0 0 0 2.9-2.9C14.5 4 11.6 1.5 8 1.5zM4.8 8h.1M6.3 4.9h.1M9.7 4.9h.1",
  info: "M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13zM8 7.2v4M8 4.8h.01",
  grip: "M6 3.5h.01M10 3.5h.01M6 8h.01M10 8h.01M6 12.5h.01M10 12.5h.01",
  chevron: "M4 6l4 4 4-4",
  plus: "M8 3v10M3 8h10",
  trash: "M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.6 9h5.8l.6-9",
  undo: "M5.5 3.5L2.5 6.5l3 3M2.5 6.5h7a4 4 0 0 1 0 8H7",
  folder: "M1.5 4a1 1 0 0 1 1-1h3.5l1.5 1.5h6a1 1 0 0 1 1 1V12a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1z",
  sparkle: "M8 2l1.4 4.6L14 8l-4.6 1.4L8 14l-1.4-4.6L2 8l4.6-1.4z",
  record: "M8 4.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7z",
  refresh: "M13.5 3.5v3h-3M2.5 12.5v-3h3M3.4 6.2A5 5 0 0 1 12.8 6M12.6 9.8A5 5 0 0 1 3.2 10",
} as const;

export type IconName = keyof typeof PATHS;

export function Icon({ name, size = 16 }: { name: IconName; size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={name === "grip" ? 2.2 : 1.3}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      <path d={PATHS[name]} />
    </svg>
  );
}
