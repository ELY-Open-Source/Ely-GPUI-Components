import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { counts, cover, filtered, sorted, split, SUBMIT, type App, type Catalog, type Sort } from "../apps";
import data from "../showcase.json";

export const catalog = data as Catalog;

/** The day an app joined, as its manifest wrote it. */
export const joined = (app: App) =>
  new Date(`${app.publishedAt.slice(0, 10)}T00:00:00Z`).toLocaleDateString("en-US", { year: "numeric", month: "short", day: "numeric", timeZone: "UTC" });

export const starCount = (n: number) => `${new Intl.NumberFormat("en-US", { notation: "compact" }).format(n)} ${n === 1 ? "star" : "stars"}`;

function Card({ app }: { app: App }) {
  return (
    <li className="app">
      <Link to="/showcase/$app/" params={{ app: app.id }}>
        <span className="app-shot">
          <img src={cover(app)} alt="" loading="lazy" />
        </span>
        <span className="app-name">
          {app.name}
          {app.building && <span className="app-tag">Building</span>}
        </span>
        <span className="app-by">{app.author}</span>
        <span className="app-text">{app.description}</span>
        <span className="app-meta">
          <span>{app.platforms.join(" · ")}</span>
          {app.stars !== undefined && <span>{starCount(app.stars)}</span>}
          <span>{joined(app)}</span>
        </span>
      </Link>
    </li>
  );
}

function Grid({ apps, label }: { apps: App[]; label: string }) {
  return (
    <ul className="apps" aria-label={label}>
      {apps.map((a) => (
        <Card key={a.id} app={a} />
      ))}
    </ul>
  );
}

export function Submit() {
  return (
    <section className="submit">
      <h2>Built something with Ely?</h2>
      <p>Add it by pull request to the showcase repository.</p>
      <a className="button" href={SUBMIT} rel="noopener">
        Submit your app
      </a>
    </section>
  );
}

export function Showcase() {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<string | null>(null);
  const [sort, setSort] = useState<Sort>("newest");
  useEffect(() => {
    document.title = "Showcase · Ely";
  }, []);
  const { featured, rest } = split(catalog);
  const searching = query.trim() !== "" || category !== null;
  const found = sorted(filtered(catalog.apps, query, category), sort);
  const empty = catalog.apps.length === 0;

  return (
    <>
      <section className="showcase">
        <h1>Showcase.</h1>
        <p className="lede">Apps built with Ely.</p>
        {empty ? (
          <p className="none">No apps yet. Yours could be the first.</p>
        ) : (
          <>
            <input className="find" type="search" placeholder={`Search ${catalog.apps.length} apps`} aria-label="Search apps" value={query} onChange={(e) => setQuery(e.target.value)} onKeyDown={(e) => e.key === "Escape" && setQuery("")} />
            <div className="filters">
              <div className="chips" role="group" aria-label="Category">
                <button className="chip" aria-pressed={category === null} onClick={() => setCategory(null)}>
                  All <span>{catalog.apps.length}</span>
                </button>
                {counts(catalog).map(([key, label, n]) => (
                  <button key={key} className="chip" aria-pressed={category === key} onClick={() => setCategory(key)}>
                    {label} <span>{n}</span>
                  </button>
                ))}
              </div>
              <div className="chips" role="group" aria-label="Sort">
                <button className="chip" aria-pressed={sort === "newest"} onClick={() => setSort("newest")}>
                  Newest
                </button>
                <button className="chip" aria-pressed={sort === "stars"} onClick={() => setSort("stars")}>
                  Most stars
                </button>
              </div>
            </div>
            {searching ? (
              found.length ? <Grid apps={found} label="Matching apps" /> : <p className="none">Nothing matches.</p>
            ) : (
              <>
                {featured.length > 0 && (
                  <>
                    <h2 className="apps-title">Featured</h2>
                    <Grid apps={featured} label="Featured apps" />
                  </>
                )}
                {rest.length > 0 && (
                  <>
                    <h2 className="apps-title">{featured.length ? "More apps" : "All apps"}</h2>
                    <Grid apps={sorted(rest, sort)} label={featured.length ? "More apps" : "All apps"} />
                  </>
                )}
              </>
            )}
          </>
        )}
      </section>
      <Submit />
    </>
  );
}
