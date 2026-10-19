import manifest from "../../examples/gallery/stories.json";

export interface Story {
  title: string;
  /** The first component alone, as pages show it. */
  name: string;
  slug: string;
  components: string[];
}

export interface Chapter {
  number: number;
  slug: string;
  title: string;
  summary: string;
  stories: Story[];
}

export const chapters: Chapter[] = manifest.pages.map((c) => ({
  ...c,
  stories: c.stories.map((s) => ({ ...s, name: s.title.split(/ \/ | · | → | \(/)[0].trim() })),
}));

export const componentCount = new Set(chapters.flatMap((c) => c.stories.flatMap((s) => s.components))).size;

/** Every story in reading order, for paging and search. */
export const all = chapters.flatMap((chapter) => chapter.stories.map((story) => ({ chapter, story })));

/** Chapters by title, else their stories by title or component. */
export function search(query: string): Chapter[] {
  const q = query.trim().toLowerCase();
  if (!q) return chapters;
  return chapters.flatMap((c) => {
    if (c.title.toLowerCase().includes(q)) return [c];
    const stories = c.stories.filter((s) => s.title.toLowerCase().includes(q) || s.components.some((n) => n.toLowerCase().includes(q)));
    return stories.length ? [{ ...c, stories }] : [];
  });
}

export const REPO = "https://github.com/ELY-Open-Source/Ely-GPUI-Components";

export const pad = (n: number) => String(n).padStart(2, "0");
