import { mark, points } from "./iso/mark";

const { width, height, polygons } = mark(0.12);

/** The E in blocks, in the page's iso colors. */
export function Mark() {
  return (
    <svg className="mark" viewBox={`0 0 ${width} ${height}`} aria-hidden="true">
      {polygons.map((p, i) => (
        <polygon key={i} className={p.accent ? `${p.face} accent` : p.face} points={points(p.points)} />
      ))}
    </svg>
  );
}
