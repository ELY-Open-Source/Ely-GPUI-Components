// Renders public/og.png from the hero; needs pnpm preview.
import { chromium } from "@playwright/test";

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1200, height: 630 }, deviceScaleFactor: 2, colorScheme: "light", reducedMotion: "reduce" });
await page.goto(process.env.SITE ?? "http://localhost:4173/");
await page.addStyleTag({ content: ".bar nav, .bar .end, .skip, .hero .button { display: none } .bar { background: none; backdrop-filter: none; padding: 0 56px } .hero { min-height: 630px; height: 630px; padding: 56px } .hero-iso { top: 0; height: 100% }" });
await page.waitForSelector(".hero-iso");
await page.waitForTimeout(800);
await page.screenshot({ path: "public/og.png", clip: { x: 0, y: 0, width: 1200, height: 630 } });
await browser.close();
console.log("og: wrote public/og.png");
