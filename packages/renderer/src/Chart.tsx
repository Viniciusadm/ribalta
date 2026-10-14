import type { Block, Ink, Shape } from "@ribaltalang/compiler";
import { useTheme } from "./Blocks";
import { lengthCss } from "./css";

type ChartBlock = Extract<Block, { type: "chart" }>;

function inkColor(ink: Ink) {
  if (ink.kind === "text") return "currentColor";
  if (ink.kind === "background") return "var(--ribalta-bg)";
  return ink.color;
}

function sectorPath({ cx, cy, outer, inner, start, end }: Extract<Shape, { shape: "sector" }>) {
  const from = (start * Math.PI) / 180;
  const full = end - start >= 360 - 1e-6;
  const sweep = full ? Math.PI * 2 - 1e-4 : ((end - start) * Math.PI) / 180;
  const to = from + sweep;
  const large = sweep > Math.PI ? 1 : 0;
  const point = (radius: number, angle: number) => `${cx + Math.cos(angle) * radius},${cy + Math.sin(angle) * radius}`;
  if (inner === 0) return `M${cx},${cy} L${point(outer, from)} A${outer},${outer} 0 ${large} 1 ${point(outer, to)} Z`;
  return `M${point(outer, from)} A${outer},${outer} 0 ${large} 1 ${point(outer, to)} L${point(inner, to)} A${inner},${inner} 0 ${large} 0 ${point(inner, from)} Z`;
}

function ShapeView({ shape }: { shape: Shape }) {
  switch (shape.shape) {
    case "rect":
      return <rect x={shape.x} y={shape.y} width={shape.w} height={shape.h} rx={shape.radius} fill={inkColor(shape.ink)} fillOpacity={shape.opacity} />;
    case "line":
      return <line x1={shape.x1} y1={shape.y1} x2={shape.x2} y2={shape.y2} stroke={inkColor(shape.ink)} strokeOpacity={shape.opacity} strokeWidth={shape.width} />;
    case "polyline":
      return (
        <polyline
          points={shape.points.map((point) => point.join(",")).join(" ")}
          fill="none"
          stroke={inkColor(shape.ink)}
          strokeOpacity={shape.opacity}
          strokeWidth={shape.width}
          strokeLinejoin="round"
          strokeLinecap="round"
        />
      );
    case "polygon":
      return <polygon points={shape.points.map((point) => point.join(",")).join(" ")} fill={inkColor(shape.ink)} fillOpacity={shape.opacity} />;
    case "circle":
      return <circle cx={shape.cx} cy={shape.cy} r={shape.r} fill={inkColor(shape.ink)} fillOpacity={shape.opacity} />;
    case "sector":
      return <path d={sectorPath(shape)} fill={inkColor(shape.ink)} stroke={inkColor(shape.stroke)} strokeWidth={shape.strokeWidth} />;
    case "text":
      return (
        <text
          x={shape.x}
          y={shape.y}
          textAnchor={shape.anchor}
          dominantBaseline={shape.baseline === "middle" ? "middle" : undefined}
          fontSize={shape.size}
          fontWeight={shape.weight === 400 ? undefined : shape.weight}
          fill={inkColor(shape.ink)}
          fillOpacity={shape.opacity === 1 ? undefined : shape.opacity}
        >
          {shape.text}
        </text>
      );
  }
}

export function Chart({ block }: { block: ChartBlock }) {
  const { scene } = block;
  const { layout } = useTheme();
  return (
    <svg
      viewBox={`0 0 ${scene.width} ${scene.height}`}
      width="100%"
      height={block.style.h ? lengthCss(block.style.h) : "100%"}
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label={block.series.map((series) => series.name).join(", ")}
      style={{ display: "block", maxHeight: block.style.h ? undefined : layout.chartMaxHeight }}
    >
      {scene.shapes.map((shape, index) => (
        <ShapeView key={index} shape={shape} />
      ))}
    </svg>
  );
}
