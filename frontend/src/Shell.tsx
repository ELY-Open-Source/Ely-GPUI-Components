import { Link, Outlet, useRouter } from "@tanstack/react-router";
import { useEffect } from "react";
import { REPO } from "./data";
import { Icon } from "./icons";
import { Mark } from "./Mark";
import { useStill } from "./iso/Iso";
import { smooth } from "./scroll";
import { setSound, useSound } from "./iso/marimba";
import { setMode, useMode } from "./theme";

function Header() {
  const theme = useMode();
  const dark = theme === "dark";
  const sound = useSound();
  return (
    <header className="bar">
      <Link to="/" className="brand">
        <Mark />
        <span>Ely</span>
      </Link>
      <nav aria-label="Site">
        <Link to="/components/" className="link" activeProps={{ "aria-current": "page" }}>
          Components
        </Link>
        <Link to="/docs/" className="link" activeProps={{ "aria-current": "page" }}>
          Docs
        </Link>
        <Link to="/showcase/" className="link" activeProps={{ "aria-current": "page" }}>
          Showcase
        </Link>
      </nav>
      <div className="end">
        <button className="tool sound" aria-label={sound ? "Sound off" : "Sound on"} aria-pressed={sound} onClick={() => setSound(!sound)}>
          <Icon name={sound ? "Sound" : "VolumeCross"} />
        </button>
        <button className="tool" aria-label={dark ? "Light mode" : "Dark mode"} onClick={() => setMode(dark ? "light" : "dark")}>
          <Icon name={dark ? "Sun" : "Moon"} />
        </button>
        <a className="tool" href={REPO} rel="noopener" aria-label="Ely on GitHub">
          <Icon name="GitHub" />
        </a>
      </div>
    </header>
  );
}

function Footer() {
  return (
    <footer className="foot">
      <span>MIT</span>
      <a href={REPO} rel="noopener">
        Source
      </a>
      <span>
        Built on <a href="https://github.com/zed-industries/zed/tree/main/crates/gpui" rel="noopener">GPUI</a>
      </span>
      <span>Geist and Geist Mono, OFL</span>
      <span>
        Icons: <a href="https://reicon.dev" rel="noopener">Reicon</a>, MIT, after Solar Icons by 480 Design,{" "}
        <a href="https://creativecommons.org/licenses/by/4.0/" rel="noopener">CC BY 4.0</a>
      </span>
    </footer>
  );
}

export function Shell() {
  const still = useStill();
  const router = useRouter();
  useEffect(() => {
    if (still) return;
    const { lenis, end } = smooth();
    // A glide in flight must not outlive its page.
    const off = router.subscribe("onBeforeNavigate", () => lenis.scrollTo(lenis.animatedScroll, { immediate: true, force: true }));
    return () => {
      off();
      end();
    };
  }, [still, router]);
  return (
    <>
      <a className="skip" href="#main">
        Skip to content
      </a>
      <Header />
      <main id="main">
        <Outlet />
      </main>
      <Footer />
    </>
  );
}
