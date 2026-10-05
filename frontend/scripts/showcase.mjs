// Writes the showcase catalog for the site: the pinned commit, or the checkout ELY_SHOWCASE names.
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { Marked } from "marked";

const REPO = "ZacharyZhang-NY/Ely-GPUI-Showcases";
const site = fileURLToPath(new URL("..", import.meta.url));
const pub = join(site, "public", "showcase");
const out = join(site, "src", "showcase.json");
const stamp = join(pub, ".commit");

/** The catalog at `commit`, unpacked with its validator's packages. */
async function fetchCatalog(commit) {
  const work = mkdtempSync(join(tmpdir(), "ely-showcase-"));
  const url = `https://codeload.github.com/${REPO}/tar.gz/${commit}`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`showcase: ${url} answered ${res.status}`);
  writeFileSync(join(work, "catalog.tar.gz"), Buffer.from(await res.arrayBuffer()));
  execFileSync("tar", ["-xzf", "catalog.tar.gz"], { cwd: work });
  const root = join(work, `${REPO.split("/")[1]}-${commit}`);
  const pnpm = process.env.npm_execpath;
  if (!pnpm) throw new Error("showcase: run through pnpm, which installs the catalog's validator");
  execFileSync(process.execPath, [pnpm, "install", "--frozen-lockfile", "--ignore-scripts", "--silent"], { cwd: root, stdio: "inherit" });
  return { root, done: () => rmSync(work, { recursive: true, force: true }) };
}

/** Checks the catalog at `root` with its own validator and writes the site's copy. */
async function write(root, source) {
  const { read } = await import(pathToFileURL(join(root, "scripts", "catalog.mjs")).href);
  const catalog = read(root);
  rmSync(pub, { recursive: true, force: true });
  mkdirSync(pub, { recursive: true });
  const apps = catalog.apps.map(({ readme, ...app }) => {
    const to = join(pub, app.id);
    mkdirSync(to, { recursive: true });
    const pictures = new Set(app.previews);
    const marked = new Marked({
      gfm: true,
      walkTokens(token) {
        if (token.type === "heading") token.depth = Math.min(6, token.depth + 1);
        if (token.type !== "image") return;
        pictures.add(token.href);
        token.href = `/showcase/${app.id}/${token.href}`;
      },
    });
    const html = readme === null ? null : marked.parse(readme, { async: false });
    for (const name of pictures) copyFileSync(join(root, "apps", app.id, name), join(to, name));
    return { ...app, html };
  });
  writeFileSync(out, `${JSON.stringify({ source, categories: catalog.categories, featured: catalog.featured, apps }, null, 2)}\n`);
  writeFileSync(stamp, source);
  console.log(`showcase: ${apps.length} apps, ${catalog.featured.length} featured, from ${source}`);
}

const local = process.env.ELY_SHOWCASE;
if (local) {
  await write(resolve(local), "local");
} else {
  const commit = readFileSync(join(site, "showcase.lock"), "utf8").trim();
  if (!/^[0-9a-f]{40}$/.test(commit)) throw new Error(`showcase: showcase.lock holds no commit: ${commit}`);
  if (existsSync(out) && existsSync(stamp) && readFileSync(stamp, "utf8") === commit) {
    console.log(`showcase: ${commit.slice(0, 7)} already written`);
  } else {
    const { root, done } = await fetchCatalog(commit);
    await write(root, commit);
    done();
  }
}
