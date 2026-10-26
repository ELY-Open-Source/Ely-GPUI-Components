import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Icon } from "./icons";
import { hold } from "./scroll";
import { mode, useMode } from "./theme";

const SPIN_AFTER = 400;
const GIVE_UP = 60000;

type State = "starting" | "slow" | "ready" | "failed";

/** One story live from the gallery, in the site's mode. */
export function Live({ page, story, title }: { page: string; story: string; title: string }) {
  const theme = useMode();
  const frame = useRef<HTMLIFrameElement>(null);
  const [state, setState] = useState<State>("starting");
  const [full, setFull] = useState(false);
  const [run, setRun] = useState(0);
  const after = useRef<HTMLSpanElement>(null);
  // Each new frame opens in the current mode.
  const start = useMemo(() => mode(), [run]);
  const src = (t: string) => `/gallery/?page=${page}&story=${story}&theme=${t}`;

  useEffect(() => {
    setState("starting");
    const slow = window.setTimeout(() => setState((s) => (s === "starting" ? "slow" : s)), SPIN_AFTER);
    const failed = window.setTimeout(() => {
      setState((s) => (s === "ready" ? s : "failed"));
      console.error(`live: ${page}/${story} sent no ready in ${GIVE_UP} ms`);
    }, GIVE_UP);
    const ready = (event: MessageEvent) => {
      if (event.source !== frame.current?.contentWindow) return;
      if (event.data?.ely === "failed") {
        clearTimeout(slow);
        clearTimeout(failed);
        setState("failed");
        console.error(`live: ${page}/${story} failed to start: ${event.data.error}`);
        return;
      }
      if (event.data?.ely !== "ready") return;
      clearTimeout(slow);
      clearTimeout(failed);
      setState("ready");
      frame.current?.contentWindow?.postMessage({ ely: "theme", theme: mode() }, location.origin);
    };
    addEventListener("message", ready);
    return () => {
      clearTimeout(slow);
      clearTimeout(failed);
      removeEventListener("message", ready);
    };
  }, [page, story, run]);

  useEffect(() => {
    frame.current?.contentWindow?.postMessage({ ely: "theme", theme }, location.origin);
  }, [theme]);

  // Before paint, so no glide frame lands after the press.
  useLayoutEffect(() => {
    if (!full) return;
    document.documentElement.style.overflow = "hidden";
    hold(true);
    const escape = (event: KeyboardEvent) => event.key === "Escape" && setFull(false);
    addEventListener("keydown", escape);
    return () => {
      document.documentElement.style.overflow = "";
      hold(false);
      removeEventListener("keydown", escape);
    };
  }, [full]);

  return (
    <>
      <button className="skip-live" onClick={() => after.current?.focus()}>
        Skip the live example
      </button>
      <div className={full ? "live full" : "live"}>
        <div className="live-tools">
          <button className="tool" aria-label="Reload the example" onClick={() => setRun((n) => n + 1)}>
            <Icon name="Refresh" />
          </button>
          <a className="tool" href={src(theme)} target="_blank" rel="noopener" aria-label="Open the example alone">
            <Icon name="ArrowRightUp" />
          </a>
          <button className="tool" aria-label={full ? "Leave full screen" : "Full screen"} onClick={() => setFull(!full)}>
            <Icon name={full ? "Minimize" : "Maximize"} />
          </button>
        </div>
        <div className="live-frame">
          <iframe key={run} ref={frame} title={`${title}, live`} src={src(start)} />
          {state !== "ready" && (
            <div className="live-state" role="status">
              {state === "slow" && <span className="spin" />}
              {state === "failed" ? (
                <>
                  Couldn't start the example.
                  <button className="text" onClick={() => setRun((n) => n + 1)}>
                    Reload
                  </button>
                </>
              ) : (
                "Starting…"
              )}
            </div>
          )}
        </div>
      </div>
      <span ref={after} tabIndex={-1} />
    </>
  );
}
