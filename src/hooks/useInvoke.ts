import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface InvokeState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
}

export function useInvoke<T>(command: string) {
  const [state, setState] = useState<InvokeState<T>>({
    data: null,
    loading: false,
    error: null,
  });

  const execute = useCallback(
    async (args?: Record<string, unknown>) => {
      setState((prev) => ({ ...prev, loading: true, error: null }));
      try {
        const result = await invoke<T>(command, args);
        setState({ data: result, loading: false, error: null });
        return result;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setState((prev) => ({ ...prev, loading: false, error: message }));
        throw err;
      }
    },
    [command]
  );

  const reset = useCallback(() => {
    setState({ data: null, loading: false, error: null });
  }, []);

  return { ...state, execute, reset };
}
