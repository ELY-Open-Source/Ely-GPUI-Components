import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { Marked, type Tokens } from "marked";

// Runs in Vite's config at build time, never in the browser.
const at = (relative: string) => fileURLToPath(new URL(relative, import.meta.url));

/** The guides in reading order, by file name. */
const ORDER = ["introduction", "installation", "getting-started", "gpui-kit", "theme", "focus", "motion", "icons", "overlays", "rendering", "i18n", "webassembly", "testing"];

export interface Guide {
  /** The path segment; the introduction has none. */
  slug: string;
  title: string;
  html: string;
  /** The `##` headings, for the on-this-page list. */
  toc: { id: string; text: string }[];
}

const escape = (text: string) => text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

const slugify = (text: string) =>
  text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

/** A heading's words: no code, emphasis or link marks. */
const plain = (markdown: string) =>
  markdown
    .replace(/`/g, "")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/(\*\*|\*)([^*]+)\1/g, "$2");

function read(file: string, files: string[]): Guide {
  const path = at(`./${file}.md`);
  files.push(path);
  const source = readFileSync(path, "utf8");
  let title = "";
  const toc: Guide["toc"] = [];
  const ids = new Set<string>();
  const marked = new Marked({
    renderer: {
      heading({ tokens, depth, text }: Tokens.Heading) {
        const words = plain(text);
        if (depth === 1) {
          if (title) throw new Error(`docs: ${file}.md has a second # title`);
          title = words;
          return "";
        }
        const html = this.parser.parseInline(tokens);
        const id = slugify(words);
        if (!id) throw new Error(`docs: ${file}.md has a heading with no id: ${words}`);
        if (ids.has(id)) throw new Error(`docs: ${file}.md repeats heading ${id}`);
        ids.add(id);
        if (depth === 2) toc.push({ id, text: words });
        return `<h${depth} id="${id}">${html}</h${depth}>`;
      },
      code({ text, lang }: Tokens.Code) {
        const [language = "", ...rest] = (lang ?? "").split(/\s+/);
        const example = rest.find((part) => part.startsWith("example="))?.slice("example=".length);
        let code = text;
        let caption = language;
        if (example) {
          if (text.trim()) throw new Error(`docs: ${file}.md puts code under example=${example}`);
          const path = at(`../../../crates/ely-gallery/examples/docs/${example}.rs`);
          files.push(path);
          code = readFileSync(path, "utf8").trimEnd();
          caption = `crates/ely-gallery/examples/docs/${example}.rs · cargo run -p ely-gallery --example docs_${example}`;
        }
        const head = caption ? `<figcaption>${escape(caption)}</figcaption>` : "";
        return `<figure class="code">${head}<button class="copy" type="button">Copy</button><pre><code>${escape(code)}</code></pre></figure>`;
      },
    },
  });
  const html = marked.parse(source, { async: false });
  if (!title) throw new Error(`docs: ${file}.md has no # title`);
  return { slug: file === "introduction" ? "" : file, title, html, toc };
}

/** Every guide, and every file read to make them. */
export function render(): { guides: Guide[]; files: string[] } {
  const files: string[] = [];
  return { guides: ORDER.map((file) => read(file, files)), files };
}
