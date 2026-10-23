import { expect, test, type Page } from "@playwright/test";
import manifest from "../../crates/ely-gallery/stories.json" with { type: "json" };

const STARTS = 45_000;

async function live(page: Page, path: string) {
  await page.goto(path);
  await expect(page.locator(".live iframe")).toHaveCount(1);
  // The host drops its state once the gallery says it is ready.
  await expect(page.locator(".live .live-state")).toHaveCount(0, { timeout: STARTS });
  return page.frameLocator(".live iframe");
}

test("a story runs live and follows the site's mode", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  const frame = await live(page, "/components/buttons/button/");
  await expect(frame.locator("canvas")).toBeVisible();
  await expect(frame.locator("html")).not.toHaveClass(/dark/);
  await page.getByRole("button", { name: "Dark mode" }).click();
  await expect(frame.locator("html")).toHaveClass(/dark/);
});

test("a frame whose wasm fetch fails once starts on its retry", async ({ page }) => {
  let aborted = false;
  await page.route("**/gallery/gallery_bg.wasm", (route) => {
    if (aborted) return route.continue();
    aborted = true;
    return route.abort();
  });
  await live(page, "/components/buttons/button/");
  expect(aborted).toBe(true);
});

test("a frame that cannot start says so at once", async ({ page }) => {
  await page.route("**/gallery/gallery_bg.wasm", (route) => route.abort());
  await page.goto("/components/buttons/button/");
  await expect(page.locator(".live .live-state")).toContainText("Couldn't start", { timeout: 15_000 });
});

test("a starting story leaves focus with the host", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  await page.goto("/");
  await page.getByRole("link", { name: "Browse" }).first().focus();
  await page.locator(".feature").first().scrollIntoViewIfNeeded();
  await expect(page.locator(".feature iframe").first()).toBeAttached();
  await expect(page.locator(".feature .live .live-state")).toHaveCount(0, { timeout: STARTS });
  await expect(page.getByRole("link", { name: "Browse" }).first()).toBeFocused();
});

test("the skip control passes the live example", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  await page.goto("/components/buttons/button/");
  const skip = page.getByRole("button", { name: "Skip the live example" });
  await skip.focus();
  await expect(skip).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(page.locator(".live + span")).toBeFocused();
});

/** Pixels darker than placeholder text inside a clip of the page. */
async function ink(page: Page, clip: { x: number; y: number; width: number; height: number }) {
  const png = (await page.screenshot({ clip })).toString("base64");
  return page.evaluate(async (src) => {
    const img = new Image();
    img.src = `data:image/png;base64,${src}`;
    await img.decode();
    const canvas = document.createElement("canvas");
    [canvas.width, canvas.height] = [img.width, img.height];
    const g = canvas.getContext("2d")!;
    g.drawImage(img, 0, 0);
    const d = g.getImageData(0, 0, img.width, img.height).data;
    let n = 0;
    for (let i = 0; i < d.length; i += 4) if (0.3 * d[i] + 0.59 * d[i + 1] + 0.11 * d[i + 2] < 90) n++;
    return n;
  }, png);
}

test("typing reaches a live field after a press", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  await page.emulateMedia({ colorScheme: "light" });
  const frame = await live(page, "/components/forms/input-textfield-clearableinput/");
  const box = (await page.locator(".live iframe").boundingBox())!;
  // The first field sits under the section's note, as the gallery lays it.
  const text = { x: box.x + 28, y: box.y + 62, width: 220, height: 20 };
  await page.mouse.click(box.x + 120, box.y + 72);
  await expect(frame.locator("textarea")).toBeFocused();
  // Its placeholder and caret are lighter than typed text.
  expect(await ink(page, text)).toBe(0);
  await page.keyboard.type("Ada");
  await expect.poll(() => ink(page, text)).toBeGreaterThan(30);
});

test("a render surface draws again when its slider moves", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  await page.emulateMedia({ colorScheme: "light" });
  await live(page, "/components/rendering/rendersurface/");
  const box = (await page.locator(".live iframe").boundingBox())!;
  // Inside the field, below the note and above the slider, as the gallery lays them.
  const field = { x: box.x + 24, y: box.y + 100, width: 720, height: 330 };
  // Ready can come a frame before the field paints.
  await expect.poll(() => ink(page, field)).toBeGreaterThan(2000);
  const before = await ink(page, field);
  await page.mouse.click(box.x + 2, box.y + 2);
  await page.keyboard.press("Tab");
  await page.keyboard.press("End");
  await expect.poll(() => ink(page, field)).not.toBe(before);
});

test("full screen covers the page and Escape leaves it", async ({ page }) => {
  await live(page, "/components/buttons/button/");
  await page.locator(".live").hover();
  await page.getByRole("button", { name: "Full screen" }).click();
  await expect(page.locator(".live")).toHaveClass(/full/);
  const viewport = page.viewportSize()!;
  expect(await page.locator(".live").boundingBox()).toEqual({ x: 0, y: 0, width: viewport.width, height: viewport.height });
  await page.keyboard.press("Escape");
  await expect(page.locator(".live")).not.toHaveClass(/full/);
});

test("home frames run only near the view", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".feature").first().locator("iframe")).toBeAttached();
  await page.locator(".close").scrollIntoViewIfNeeded();
  await expect(page.locator(".feature iframe")).toHaveCount(0);
});

// One story from every fifth chapter, each drawn in its frame.
for (const chapter of manifest.pages.filter((_, i) => i % 5 === 0)) {
  test(`the first ${chapter.title} story draws`, async ({ page }) => {
    const frame = await live(page, `/components/${chapter.slug}/`);
    const shot = await frame.locator("canvas").screenshot();
    expect(shot.length).toBeGreaterThan(4000);
  });
}
