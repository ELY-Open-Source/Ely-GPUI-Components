import { bounds, faces, painted, type Block, type Point } from "./engine.ts";

const b = (x: number, z: number, w: number, d: number, h: number, y = 0, accent = false): Block => ({ x, z, w, d, h, y, accent, beat: 0 });

/** The letter E in blocks, its middle arm in the accent. */
const BLOCKS = [b(0, 0, 1, 1, 5), b(1, 0, 2.4, 1, 1), b(1, 0, 1.8, 1, 1, 2, true), b(1, 0, 2.4, 1, 1, 4)];

export type Face = "top" | "front" | "right";

export interface Polygon {
  points: Point[];
  face: Face;
  accent: boolean;
}

/** The mark's faces back to front, in a box `pad` larger than the mark each side. */
export function mark(pad: number): { width: number; height: number; polygons: Polygon[] } {
  const [x0, y0, x1, y1] = bounds(BLOCKS);
  const [ox, oy] = [pad - x0, pad - y0];
  const polygons = painted(BLOCKS).flatMap((k) => {
    const f = faces(k, k.h);
    return (["right", "front", "top"] as const).map((face) => ({ points: f[face].map(([x, y]): Point => [x + ox, y + oy]), face, accent: !!k.accent }));
  });
  return { width: x1 - x0 + pad * 2, height: y1 - y0 + pad * 2, polygons };
}

export const points = (p: Point[]) => p.map(([x, y]) => `${+x.toFixed(3)},${+y.toFixed(3)}`).join(" ");
