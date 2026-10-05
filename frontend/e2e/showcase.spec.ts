import { expect, test, type Page } from "@playwright/test";
import { SUBMIT, type Catalog } from "../src/apps";
import data from "../src/showcase.json" with { type: "json" };

// The catalog the site was built with: the pinned commit, or ELY_SHOWCASE's checkout.
const catalog = data as Catalog;
const empty = catalog.apps.length === 0;

function watch(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
  page.on("pageerror", (e) => errors.push(e.message));
  return errors;
}

test("the header's Showcase link opens the page, with a way to submit", async ({ page }) => {
  const errors = watch(page);
  await page.goto("/components/");
  await page.getByRole("navigation", { name: "Site" }).getByRole("link", { name: "Showcase" }).click();
  await expect(page).toHaveURL("/showcase/");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Showcase.");
  await expect(page).toHaveTitle("Showcase · Ely");
  await expect(page.getByRole("link", { name: "Submit your app" })).toHaveAttribute("href", SUBMIT);
  expect(errors).toEqual([]);
});

test("an empty catalog says so", async ({ page }) => {
  test.skip(!empty, "the catalog has apps");
  await page.goto("/showcase/");
  await expect(page.locator(".none")).toHaveText("No apps yet. Yours could be the first.");
  await expect(page.getByRole("searchbox")).toHaveCount(0);
});

test("featured apps come first, then the rest, and categories count them", async ({ page }) => {
  test.skip(empty, "the catalog is empty");
  await page.goto("/showcase/");
  const all = page.getByRole("group", { name: "Category" }).getByRole("button");
  await expect(all.first()).toHaveText(`All ${catalog.apps.length}`);
  await expect(all).toHaveCount(Object.keys(catalog.categories).length + 1);
  if (catalog.featured.length) {
    const first = catalog.apps.find((a) => a.id === catalog.featured[0])!;
    await expect(page.getByRole("list", { name: "Featured apps" }).locator(".app-name").first()).toContainText(first.name);
  }
});

test("search narrows the apps, and a miss says so", async ({ page }) => {
  test.skip(empty, "the catalog is empty");
  const [app] = catalog.apps;
  await page.goto("/showcase/");
  const find = page.getByRole("searchbox", { name: "Search apps" });
  await find.fill(app.name);
  await expect(page.getByRole("list", { name: "Matching apps" }).locator(".app-name").first()).toContainText(app.name);
  await find.fill("zzzz-no-such-app");
  await expect(page.locator(".none")).toHaveText("Nothing matches.");
  await find.press("Escape");
  await expect(find).toHaveValue("");
});

test("a card opens its app, with its previews and links", async ({ page }) => {
  test.skip(empty, "the catalog is empty");
  const errors = watch(page);
  const [app] = catalog.apps;
  await page.goto("/showcase/");
  await page.locator(`.app a[href="/showcase/${app.id}/"]`).first().click();
  await expect(page).toHaveURL(`/showcase/${app.id}/`);
  await expect(page.getByRole("heading", { level: 1 })).toContainText(app.name);
  const shots = page.locator(".app-previews img");
  await expect(shots).toHaveCount(app.previews.length);
  await expect.poll(() => shots.first().evaluate((img: HTMLImageElement) => img.naturalWidth)).toBeGreaterThan(0);
  const links = page.locator(".app-links");
  if (app.source) await expect(links.getByRole("link", { name: "Source" })).toHaveAttribute("href", app.source);
  if (app.website) await expect(links.getByRole("link", { name: "Website" })).toHaveAttribute("href", app.website);
  await expect(page.locator(".app-readme")).toHaveCount(app.html ? 1 : 0);
  expect(errors).toEqual([]);
});

test("an unknown app says so", async ({ page }) => {
  const errors = watch(page);
  // Ids are kebab-case, so no catalog holds this one.
  await page.goto("/showcase/no_such_app/");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Not found");
  expect(errors.join()).toContain("No app named no_such_app");
});
