import { useEffect, useState } from "react";
import { api, type InstallProgress } from "../lib/api";

export const ACTIVE = ["resolving", "downloading", "verifying", "extracting", "installing"];

/** Live install state, fed by backend events. Lives at app level so every page sees the same data. */
export function useInstalls() {
  const [progress, setProgress] = useState<Record<string, InstallProgress>>({});
  useEffect(() => {
    let off = () => {};
    api.onProgress((p) => setProgress((cur) => ({ ...cur, [p.id]: p }))).then((u) => (off = u));
    return () => off();
  }, []);
  const dismiss = (id: string) => setProgress((cur) => { const n = { ...cur }; delete n[id]; return n; });
  return { progress, dismiss };
}
