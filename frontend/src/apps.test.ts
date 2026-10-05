import assert from "node:assert/strict";
import { test } from "node:test";
import { counts, filtered, sorted, split, type App, type Catalog } from "./apps.ts";

const app = (id: string, publishedAt: string, more: Partial<App> = {}): App => ({
  id,
  name: id,
  author: "Ada",
  description: `${id} does things.`,
  category: "dev",
  platforms: ["Linux"],
  website: null,
  source: `https://github.com/ada/${id}`,
  publishedAt,
  previews: ["preview0.png"],
  html: null,
  ...more,
});

const old = app("old", "2026-01-01T00:00:00Z", { stars: 900 });
const mid = app("mid", "2026-05-01T00:00:00+08:00", { category: "work", author: "Grace" });
const fresh = app("fresh", "2026-09-01T00:00:00Z", { stars: 5 });
const twin = app("twin", "2026-09-01T00:00:00Z", { stars: 50 });
const catalog: Catalog = { source: "test", categories: { dev: "Developer Tools", work: "Productivity & Media", system: "System & Desktop" }, featured: ["mid", "old"], apps: [old, mid, fresh, twin] };

test("newest comes first, and stars break a tie", () => {
  assert.deepEqual(sorted(catalog.apps, "newest").map((a) => a.id), ["twin", "fresh", "mid", "old"]);
});

test("most stars comes first, and an app without stars comes last", () => {
  assert.deepEqual(sorted(catalog.apps, "stars").map((a) => a.id), ["old", "twin", "fresh", "mid"]);
});

test("featured apps keep the catalog's order, and the rest leave them out", () => {
  const { featured, rest } = split(catalog);
  assert.deepEqual(featured.map((a) => a.id), ["mid", "old"]);
  assert.deepEqual(rest.map((a) => a.id), ["fresh", "twin"]);
});

test("a featured id with no app fails", () => {
  assert.throws(() => split({ ...catalog, featured: ["ghost"] }), /featured ghost is no app/);
});

test("search reads the name, the author and the description, in any case", () => {
  assert.deepEqual(filtered(catalog.apps, "FRESH", null).map((a) => a.id), ["fresh"]);
  assert.deepEqual(filtered(catalog.apps, "grace", null).map((a) => a.id), ["mid"]);
  assert.deepEqual(filtered(catalog.apps, "twin does", null).map((a) => a.id), ["twin"]);
  assert.deepEqual(filtered(catalog.apps, "  ", null).length, 4);
});

test("a category narrows the search", () => {
  assert.deepEqual(filtered(catalog.apps, "", "work").map((a) => a.id), ["mid"]);
  assert.deepEqual(filtered(catalog.apps, "fresh", "work"), []);
});

test("every category counts its apps, empty ones too", () => {
  assert.deepEqual(counts(catalog), [["dev", "Developer Tools", 3], ["work", "Productivity & Media", 1], ["system", "System & Desktop", 0]]);
});
