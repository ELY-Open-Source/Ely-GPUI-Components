import { expect, test, type Page } from "@playwright/test";
import manifest from "../../examples/gallery/stories.json" with { type: "json" };

/** A story's short name, as `src/data.ts` derives it. */
const name = (title: string) => title.split(/ \/ | · | → | \(/)[0].trim();

/** Collects console errors and uncaught exceptions. */
function watch(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
  page.on("pageerror", (e) => errors.push(e.message));
  return errors;
}

test("the home page shows the hero, five features, the city and the credits", async ({ page }) => {
  const errors = watch(page);
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Components for GPUI.");
  await expect(page.locator(".hero-iso")).toBeVisible();
  await expect(page.locator(".feature")).toHaveCount(5);
  await expect(page.locator(".city-names li")).toHaveCount(manifest.pages.length);
  await expect(page.locator(".foot")).toContainText("Solar Icons by 480 Design");
  expect(errors).toEqual([]);
});

test("the mode toggles, and a reload keeps it", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await page.goto("/components/");
  const html = page.locator("html");
  await expect(html).not.toHaveClass(/dark/);
  await page.getByRole("button", { name: "Dark mode" }).click();
  await expect(html).toHaveClass(/dark/);
  await page.reload();
  await expect(html).toHaveClass(/dark/);
});

test("the index lists every chapter with its stories", async ({ page }) => {
  await page.goto("/components/");
  const rows = page.locator(".chapters li");
  await expect(rows).toHaveCount(manifest.pages.length);
  const [first] = manifest.pages;
  await expect(rows.first().locator(".titles a")).toHaveCount(first.stories.length);
});

test("search filters by story and component name, and Enter opens the first match", async ({ page }) => {
  await page.goto("/components/");
  const find = page.getByRole("searchbox", { name: "Search components" });
  await find.fill("zzzz");
  await expect(page.locator(".none")).toContainText("Nothing matches");
  await find.fill("breadcrumb");
  await expect(page.locator(".chapters li").first()).toContainText(/Breadcrumb/);
  await find.press("Enter");
  await expect(page).toHaveURL(/\/components\/[a-z_]+\/.*breadcrumb/);
});

test("Slash focuses search from the page", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  await page.goto("/components/");
  await page.keyboard.press("/");
  await expect(page.getByRole("searchbox", { name: "Search components" })).toBeFocused();
});

test("a chapter opens its first story, and the pager and arrows walk on", async ({ page, isMobile }) => {
  const [chapter] = manifest.pages;
  await page.goto(`/components/${chapter.slug}/`);
  await expect(page).toHaveURL(`/components/${chapter.slug}/${chapter.stories[0].slug}/`);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(name(chapter.stories[0].title));
  await page.locator(".pager a.next").click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(name(chapter.stories[1].title));
  if (isMobile) return;
  await page.keyboard.press("ArrowLeft");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(name(chapter.stories[0].title));
});

test("the rail marks the current story", async ({ page, isMobile }) => {
  test.skip(isMobile, "the rail hides on phones");
  const chapter = manifest.pages[manifest.pages.length - 1];
  const story = chapter.stories[chapter.stories.length - 1];
  await page.goto(`/components/${chapter.slug}/${story.slug}/`);
  await expect(page.locator('.side li [aria-current="page"]')).toHaveText(name(story.title));
  await expect(page.locator('.side li [aria-current="page"]')).toBeInViewport();
});

test("a chapter's name in the city names it and opens it", async ({ page, isMobile }) => {
  test.skip(isMobile, "hover");
  await page.goto("/");
  const chapter = manifest.pages[15];
  await page.locator(".city-names a", { hasText: chapter.title }).first().hover();
  await expect(page.locator(".city h2")).toHaveText(`${chapter.title}.`);
  await page.locator(".city-names a", { hasText: chapter.title }).first().click();
  await expect(page).toHaveURL(`/components/${chapter.slug}/${chapter.stories[0].slug}/`);
});

test("an unknown chapter says so", async ({ page }) => {
  const errors = watch(page);
  await page.goto("/components/nowhere/");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Not found");
  expect(errors.join()).toContain("No chapter");
});

test("a story the web cannot run shows its native capture, in the site's mode", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await page.goto("/components/forms/pathinput/");
  const shot = page.locator("img.shot");
  await expect(shot).toHaveAttribute("srcset", /pathinput-light\.jpg/);
  await expect(page.locator(".live iframe")).toHaveCount(0);
  await page.getByRole("button", { name: "Dark mode" }).click();
  await expect(shot).toHaveAttribute("srcset", /pathinput-dark\.jpg/);
  await expect.poll(() => shot.evaluate((img: HTMLImageElement) => img.naturalWidth)).toBeGreaterThan(0);
});

test("a live story that opens a window shows that window natively", async ({ page }) => {
  await page.goto("/components/shell/aboutdialog/");
  await expect(page.locator(".live iframe")).toHaveCount(1);
  const shot = page.locator(".windows img.shot");
  await expect(shot).toHaveAttribute("srcset", /shell-about-window/);
  await shot.scrollIntoViewIfNeeded();
  await expect.poll(() => shot.evaluate((img: HTMLImageElement) => img.naturalWidth)).toBeGreaterThan(0);
});

test("the header leads with the mark, and the home link keeps its name", async ({ page }) => {
  await page.goto("/components/");
  const home = page.getByRole("link", { name: "Ely", exact: true });
  await expect(home.locator("svg.mark polygon")).toHaveCount(12);
  await expect(home.locator("svg.mark .accent")).toHaveCount(3);
  await home.click();
  await expect(page).toHaveURL("/");
});

test("the favicon and the touch icon are served", async ({ page, request }) => {
  await page.goto("/components/");
  for (const [rel, type] of [["icon", "image/svg+xml"], ["apple-touch-icon", "image/png"]]) {
    const href = await page.locator(`link[rel="${rel}"]`).getAttribute("href");
    const res = await request.get(href!);
    expect(res.status()).toBe(200);
    expect(res.headers()["content-type"]).toContain(type);
  }
});
