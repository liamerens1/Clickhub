import { useCallback, useEffect, useRef, useState } from "react";

import {
  type ConfigSnapshot,
  getConfig,
  type IpcResult,
  onConfigChanged,
  reloadConfig,
  saveConfig,
} from "../shared/ipc";
import type { Config } from "../shared/schema";

/** How many steps Undo can go back in one session. */
const UNDO_LIMIT = 50;

export type SaveState =
  { kind: "idle" } | { kind: "saving" } | { kind: "saved" } | { kind: "error"; message: string };

export interface ConfigStore {
  /** The config being edited; `null` until the first load. */
  config: Config | null;
  snapshot: ConfigSnapshot | null;
  loadError: string | null;
  save: SaveState;
  canUndo: boolean;
  /** Applies `next` at once and saves it. A failed save is reported and rolled back. */
  commit: (next: Config) => void;
  undo: () => void;
  /** Re-reads `config.json` from disk (after a hand edit). */
  reload: () => void;
}

const same = (a: Config | null, b: Config | null) => JSON.stringify(a) === JSON.stringify(b);

/**
 * The settings window's copy of the config. Every change applies to the live ring and is saved
 * straight away (in order, one save at a time); Undo steps back through this session's changes.
 */
export function useConfigStore(): ConfigStore {
  const [config, setConfig] = useState<Config | null>(null);
  const [snapshot, setSnapshot] = useState<ConfigSnapshot | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [save, setSave] = useState<SaveState>({ kind: "idle" });
  const [undoDepth, setUndoDepth] = useState(0);

  const current = useRef<Config | null>(null);
  /** The last config Rust confirmed as running. */
  const confirmed = useRef<Config | null>(null);
  const pending = useRef(0);
  const queue = useRef<Promise<void>>(Promise.resolve());
  /** Configs to step back to, oldest first. */
  const history = useRef<Config[]>([]);

  const adopt = useCallback((next: ConfigSnapshot) => {
    confirmed.current = next.config;
    setSnapshot(next);
    if (pending.current === 0 && !same(current.current, next.config)) {
      current.current = next.config;
      setConfig(next.config);
    }
  }, []);

  const applyRead = useCallback(
    (result: IpcResult<ConfigSnapshot>) => {
      if (result.ok) {
        setLoadError(null);
        adopt(result.value);
      } else {
        setLoadError(result.error);
      }
    },
    [adopt],
  );

  useEffect(() => {
    void getConfig().then(applyRead);
    const unlisten = onConfigChanged(() => {
      if (pending.current === 0) void getConfig().then(applyRead);
    });
    return () => {
      void unlisten.then((stop) => {
        stop();
      });
    };
  }, [applyRead]);

  const persist = useCallback(
    (next: Config, recordUndo: boolean) => {
      const previous = current.current;
      if (previous === null || same(previous, next)) return;
      current.current = next;
      setConfig(next);
      if (recordUndo) {
        history.current = [...history.current.slice(1 - UNDO_LIMIT), previous];
        setUndoDepth(history.current.length);
      }
      pending.current += 1;
      setSave({ kind: "saving" });
      queue.current = queue.current.then(async () => {
        const result = await saveConfig(next);
        pending.current -= 1;
        if (result.ok) {
          adopt(result.value);
          if (pending.current === 0) setSave({ kind: "saved" });
          return;
        }
        setSave({ kind: "error", message: result.error });
        if (recordUndo && history.current[history.current.length - 1] === previous) {
          history.current = history.current.slice(0, -1);
          setUndoDepth(history.current.length);
        }
        if (pending.current === 0 && confirmed.current !== null) {
          current.current = confirmed.current;
          setConfig(confirmed.current);
        }
      });
    },
    [adopt],
  );

  const commit = useCallback(
    (next: Config) => {
      persist(next, true);
    },
    [persist],
  );

  const undo = useCallback(() => {
    const previous = history.current[history.current.length - 1];
    if (previous === undefined) return;
    history.current = history.current.slice(0, -1);
    setUndoDepth(history.current.length);
    persist(previous, false);
  }, [persist]);

  const reload = useCallback(() => {
    void reloadConfig().then(applyRead);
  }, [applyRead]);

  return {
    config,
    snapshot,
    loadError,
    save,
    canUndo: undoDepth > 0,
    commit,
    undo,
    reload,
  };
}
