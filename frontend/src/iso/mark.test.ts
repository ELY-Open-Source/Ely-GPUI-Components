import assert from "node:assert/strict";
import { test } from "node:test";
import { mark, points } from "./mark.ts";

test("the mark draws three faces of each of its four blocks, the middle arm's in the accent", () => {
  const { polygons } = mark(0);
  assert.equal(polygons.length, 12);
  assert.equal(polygons.filter((p) => p.accent).length, 3);
  assert.deepEqual(polygons.slice(0, 3).map((p) => p.face), ["right", "front", "top"]);
});

test("the mark fills its box less the padding", () => {
  const pad = 0.5;
  const { width, height, polygons } = mark(pad);
  const all = polygons.flatMap((p) => p.points);
  const [xs, ys] = [all.map(([x]) => x), all.map(([, y]) => y)];
  assert.ok(Math.abs(Math.min(...xs) - pad) < 1e-9 && Math.abs(Math.max(...xs) - (width - pad)) < 1e-9);
  assert.ok(Math.abs(Math.min(...ys) - pad) < 1e-9 && Math.abs(Math.max(...ys) - (height - pad)) < 1e-9);
  assert.ok(height > width, "the E stands taller than wide");
});

test("points print as an SVG list", () => {
  assert.equal(points([[1, 2.5], [0.12345, 3]]), "1,2.5 0.123,3");
});
