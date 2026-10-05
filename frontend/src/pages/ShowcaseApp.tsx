import { Link, useParams } from "@tanstack/react-router";
import { useEffect } from "react";
import { catalog, joined, starCount, Submit } from "./Showcase";
import { NotFound } from "./NotFound";

export function ShowcaseApp() {
  const { app: id } = useParams({ strict: false });
  const app = catalog.apps.find((a) => a.id === id);
  useEffect(() => {
    if (app) document.title = `${app.name} · Showcase · Ely`;
  }, [app]);
  if (!app) return <NotFound what={`No app named ${id} in the showcase.`} />;

  return (
    <>
      <article className="app-page">
        <Link className="crumb" to="/showcase/">
          ← Showcase
        </Link>
        <h1>
          {app.name}
          {app.building && <span className="app-tag">Building</span>}
        </h1>
        <p className="app-by">{app.author}</p>
        <p className="lede">{app.description}</p>
        <div className="app-links">
          {app.website && (
            <a className="button" href={app.website} rel="noopener">
              Website
            </a>
          )}
          {app.source && (
            <a className={app.website ? "text" : "button"} href={app.source} rel="noopener">
              Source
            </a>
          )}
        </div>
        <dl className="app-facts">
          <div>
            <dt>Category</dt>
            <dd>{catalog.categories[app.category]}</dd>
          </div>
          <div>
            <dt>Platforms</dt>
            <dd>{app.platforms.join(", ")}</dd>
          </div>
          {app.stars !== undefined && (
            <div>
              <dt>GitHub</dt>
              <dd>{starCount(app.stars)}</dd>
            </div>
          )}
          <div>
            <dt>Added</dt>
            <dd>{joined(app)}</dd>
          </div>
        </dl>
        <div className="app-previews">
          {app.previews.map((name, i) => (
            <img key={name} src={`/showcase/${app.id}/${name}`} alt={`${app.name}, screenshot ${i + 1} of ${app.previews.length}`} loading={i ? "lazy" : "eager"} />
          ))}
        </div>
        {app.html && <div className="prose app-readme" dangerouslySetInnerHTML={{ __html: app.html }} />}
      </article>
      <Submit />
    </>
  );
}
