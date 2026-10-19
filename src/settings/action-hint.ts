import type { Action } from "../shared/schema";

/** One-line description of what a slot does, for the slot list. */
export function actionHint(action: Action): string {
  switch (action.kind) {
    case "shortcut":
      return action.keys;
    case "launch": {
      const name = action.path.split(/[\\/]/).pop() ?? action.path;
      return `Open ${name === "" ? action.path : name}`;
    }
    case "url": {
      const host = /^https?:\/\/([^/?#]+)/i.exec(action.url)?.[1];
      return `Open ${host ?? action.url}`;
    }
  }
}
