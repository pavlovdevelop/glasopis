import { useEffect, useState } from "react";
import { api, events, type Status } from "../services/api";

/** Следи състоянието на диктовката и нивото на микрофона. */
export function useStatus(): { status: Status; level: number } {
  const [status, setStatus] = useState<Status>({ state: "idle" });
  const [level, setLevel] = useState(0);

  useEffect(() => {
    let mounted = true;
    const unlisteners: Array<() => void> = [];

    void api
      .getStatus()
      .then((current) => mounted && setStatus(current))
      .catch(() => undefined);

    void events.onStatus((next) => mounted && setStatus(next)).then((un) => unlisteners.push(un));
    void events.onLevel((next) => mounted && setLevel(next)).then((un) => unlisteners.push(un));

    return () => {
      mounted = false;
      unlisteners.forEach((un) => un());
    };
  }, []);

  return { status, level };
}
