// Writes public/favicon.svg, public/apple-touch-icon.png and the repo's logo.png from the header's mark.
import { chromium } from "@playwright/test";
import { readFileSync, writeFileSync } from "node:fs";
import { mark, points } from "../src/iso/mark.ts";

const css = readFileSync("src/styles.css", "utf8");

/** The iso colors a `:root` block of styles.css sets, as the header's mark wears them. */
function colors(selector) {
  const start = `\n${css}`.indexOf(`\n${selector} {`);
  const block = start < 0 ? null : css.slice(start, css.indexOf("}", start));
  if (!block) throw new Error(`favicon: styles.css has no ${selector} block`);
  const v = (name) => {
    const value = block.match(new RegExp(`--${name}: (#[0-9a-f]{6});`))?.[1];
    if (!value) throw new Error(`favicon: ${selector} sets no --${name}`);
    return value;
  };
  return { top: v("bg"), front: v("iso-side"), right: v("iso-shade"), line: v("iso-line"), accent: v("accent") };
}
const [LIGHT, DARK] = [colors(":root"), colors(":root.dark")];

const rules = (c) => `.top{fill:${c.top}}.front{fill:${c.front}}.right{fill:${c.right}}polygon{stroke:${c.line}}.accent{stroke:${c.accent}}`;

/** The mark centered in a square, `inset` of the side left round it. */
function svg(inset, stroke, style, back = "") {
  const { width, height, polygons } = mark(0);
  const side = Math.max(width, height) / (1 - inset * 2);
  const [ox, oy] = [(side - width) / 2, (side - height) / 2];
  const body = polygons.map((p) => `<polygon class="${p.accent ? `${p.face} accent` : p.face}" points="${points(p.points.map(([x, y]) => [x + ox, y + oy]))}"/>`).join("");
  const fill = back && `<rect width="${side}" height="${side}" fill="${back}"/>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${+side.toFixed(3)} ${+side.toFixed(3)}" stroke-width="${stroke}" stroke-linejoin="round"><style>${style}</style>${fill}${body}</svg>\n`;
}

writeFileSync("public/favicon.svg", svg(0.04, 0.28, `${rules(LIGHT)}@media (prefers-color-scheme:dark){${rules(DARK)}}`));

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 180, height: 180 } });
await page.setContent(`<style>body{margin:0}svg{display:block;width:180px;height:180px}</style>${svg(0.18, 0.16, rules(LIGHT), LIGHT.top)}`);
await page.screenshot({ path: "public/apple-touch-icon.png" });
await page.setViewportSize({ width: 2048, height: 2048 });
await page.setContent(`<style>body{margin:0;background:none}svg{display:block;width:2048px;height:2048px}</style>${svg(0.04, 0.1, rules(LIGHT))}`);
await page.screenshot({ path: "../logo.png", omitBackground: true });
await browser.close();
console.log("favicon: wrote public/favicon.svg, public/apple-touch-icon.png and ../logo.png");
