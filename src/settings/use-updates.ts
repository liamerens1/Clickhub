import { useCallback, useEffect, useState } from "react";

import {
  checkForUpdates,
  getUpdateState,
  installUpdate,
  onUpdateChanged,
  type UpdateState,
} from "../shared/ipc";

export interface Updates {
  state: UpdateState;
  check: () => void;
  install: () => void;
}

/** Where updating stands, kept current by `update:changed` from Rust. */
export function useUpdates(): Updates {
  const [state, setState] = useState<UpdateState>({ kind: "idle" });

  useEffect(() => {
    void getUpdateState().then((result) => {
      if (result.ok) setState(result.value);
    });
    const unlisten = onUpdateChanged(setState);
    return () => {
      void unlisten.then((stop) => {
        stop();
      });
    };
  }, []);

  const check = useCallback(() => {
    void checkForUpdates().then((result) => {
      setState(result.ok ? result.value : { kind: "failed", error: result.error });
    });
  }, []);

  const install = useCallback(() => {
    void installUpdate().then((result) => {
      if (!result.ok) setState({ kind: "failed", error: result.error });
    });
  }, []);

  return { state, check, install };
}
