/** The keys of a shortcut ("Ctrl+Shift+S") as keycaps. */
export function Keys({ shortcut }: { shortcut: string }) {
  return (
    <span className="keys">
      {shortcut.split("+").map((part, index) => (
        // Parts repeat only in invalid input; the index keeps the keys unique anyway.
        <kbd key={`${String(index)}-${part}`} className="kbd">
          {part}
        </kbd>
      ))}
    </span>
  );
}
