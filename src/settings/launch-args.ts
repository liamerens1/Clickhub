/**
 * Turns the single "Arguments" text box into the `args` list of a launch action and back.
 * Arguments are separated by spaces; double quotes keep spaces inside one argument.
 */

export function parseArgs(text: string): string[] {
  const args: string[] = [];
  let current = "";
  let quoted = false;
  let started = false;
  for (const char of text) {
    if (char === '"') {
      quoted = !quoted;
      started = true;
    } else if (/\s/.test(char) && !quoted) {
      if (started) args.push(current);
      current = "";
      started = false;
    } else {
      current += char;
      started = true;
    }
  }
  if (started) args.push(current);
  return args;
}

export function formatArgs(args: readonly string[]): string {
  return args.map((arg) => (arg === "" || /[\s"]/.test(arg) ? `"${arg}"` : arg)).join(" ");
}
