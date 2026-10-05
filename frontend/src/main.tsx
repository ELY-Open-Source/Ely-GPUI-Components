import "@fontsource-variable/geist";
import "@fontsource-variable/geist-mono";
import { createRootRoute, createRoute, createRouter, redirect, RouterProvider } from "@tanstack/react-router";
import { createRoot } from "react-dom/client";
import { Shell } from "./Shell";
import { chapters } from "./data";
import { Chapters } from "./pages/Chapters";
import { Docs } from "./pages/Docs";
import { Home } from "./pages/Home";
import { NotFound } from "./pages/NotFound";
import { Showcase } from "./pages/Showcase";
import { ShowcaseApp } from "./pages/ShowcaseApp";
import { Story } from "./pages/Story";
import "./styles.css";

const root = createRootRoute({ component: Shell, notFoundComponent: () => <NotFound /> });
const home = createRoute({ getParentRoute: () => root, path: "/", component: Home });
const index = createRoute({ getParentRoute: () => root, path: "/components", component: Chapters });
const chapter = createRoute({
  getParentRoute: () => root,
  path: "/components/$page",
  // A chapter opens its first story.
  beforeLoad: ({ params }) => {
    const found = chapters.find((c) => c.slug === params.page);
    if (found) throw redirect({ to: "/components/$page/$story/", params: { page: found.slug, story: found.stories[0].slug } });
  },
  component: () => <NotFound what="No chapter by that name." />,
});
const story = createRoute({ getParentRoute: () => root, path: "/components/$page/$story", component: Story });
const docs = createRoute({ getParentRoute: () => root, path: "/docs", component: Docs });
const doc = createRoute({ getParentRoute: () => root, path: "/docs/$page", component: Docs });
const showcase = createRoute({ getParentRoute: () => root, path: "/showcase", component: Showcase });
const showcaseApp = createRoute({ getParentRoute: () => root, path: "/showcase/$app", component: ShowcaseApp });

const router = createRouter({
  routeTree: root.addChildren([home, index, chapter, story, docs, doc, showcase, showcaseApp]),
  trailingSlash: "always",
  scrollRestoration: true,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

createRoot(document.getElementById("root")!).render(<RouterProvider router={router} />);
