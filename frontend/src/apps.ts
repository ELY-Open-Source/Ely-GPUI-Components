/** An app from the showcase catalog, as `scripts/showcase.mjs` writes it. */
export interface App {
  id: string;
  name: string;
  author: string;
  description: string;
  category: string;
  platforms: string[];
  website: string | null;
  source: string | null;
  publishedAt: string;
  previews: string[];
  building?: boolean;
  stars?: number;
  starsUpdatedAt?: string;
  /** The README, rendered; null without one. */
  html: string | null;
}

export interface Catalog {
  source: string;
  categories: Record<string, string>;
  featured: string[];
  apps: App[];
}

export type Sort = "newest" | "stars";

export const SUBMIT = "https://github.com/ZacharyZhang-NY/Ely-GPUI-Showcases#submit-an-app";

/** Newest first, then most stars; or most stars, then newest. */
export function sorted(apps: App[], sort: Sort): App[] {
  const time = (a: App) => Date.parse(a.publishedAt);
  const stars = (a: App) => a.stars ?? -1;
  const [first, second] = sort === "newest" ? [time, stars] : [stars, time];
  return [...apps].sort((a, b) => first(b) - first(a) || second(b) - second(a) || a.name.localeCompare(b.name));
}

/** The featured apps in their order, and the rest. */
export function split(catalog: Catalog): { featured: App[]; rest: App[] } {
  const byId = new Map(catalog.apps.map((a) => [a.id, a]));
  const featured = catalog.featured.map((id) => {
    const app = byId.get(id);
    if (!app) throw new Error(`showcase: featured ${id} is no app`);
    return app;
  });
  return { featured, rest: catalog.apps.filter((a) => !catalog.featured.includes(a.id)) };
}

/** Apps in a category, or all with none, whose name, author or description holds the query. */
export function filtered(apps: App[], query: string, category: string | null): App[] {
  const q = query.trim().toLowerCase();
  return apps.filter((a) => (!category || a.category === category) && (!q || [a.name, a.author, a.description].some((s) => s.toLowerCase().includes(q))));
}

/** How many apps each category holds. */
export function counts(catalog: Catalog): [string, string, number][] {
  return Object.entries(catalog.categories).map(([key, label]) => [key, label, catalog.apps.filter((a) => a.category === key).length]);
}

export const cover = (app: App) => `/showcase/${app.id}/${app.previews[0]}`;
