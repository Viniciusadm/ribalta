use std::f64::consts::{FRAC_PI_2, TAU};

use crate::ir::{ChartKind, ChartScene, Ink, Series, Shape, TextAnchor, TextBaseline};

const WIDTH: f64 = 1200.0;
const HEIGHT: f64 = 600.0;
const LEGEND_HEIGHT: f64 = 64.0;
const TEXT_SIZE: f64 = 26.0;
const MARGIN_TOP: f64 = 24.0;
const MARGIN_RIGHT: f64 = 24.0;
const MARGIN_BOTTOM: f64 = 72.0;
const MARGIN_LEFT: f64 = 96.0;

pub struct ChartData<'a> {
    pub kind: ChartKind,
    pub labels: &'a [String],
    pub series: &'a [Series],
    pub legend: bool,
    pub stacked: bool,
    pub values: bool,
    pub palette: &'a [String],
}

fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn color(value: &str) -> Ink {
    Ink::Color { color: value.to_string() }
}

struct TextStyle {
    anchor: TextAnchor,
    baseline: TextBaseline,
    ink: Ink,
    opacity: f64,
    weight: u16,
}

fn axis_style() -> TextStyle {
    TextStyle { anchor: TextAnchor::End, baseline: TextBaseline::Middle, ink: Ink::Text, opacity: 0.6, weight: 400 }
}

fn category_style() -> TextStyle {
    TextStyle { anchor: TextAnchor::Middle, baseline: TextBaseline::Alphabetic, ink: Ink::Text, opacity: 0.75, weight: 400 }
}

fn value_style() -> TextStyle {
    TextStyle { anchor: TextAnchor::Middle, baseline: TextBaseline::Alphabetic, ink: Ink::Text, opacity: 1.0, weight: 700 }
}

fn percent_style() -> TextStyle {
    TextStyle { anchor: TextAnchor::Middle, baseline: TextBaseline::Middle, ink: color("#ffffff"), opacity: 1.0, weight: 700 }
}

fn legend_style() -> TextStyle {
    TextStyle { anchor: TextAnchor::Start, baseline: TextBaseline::Middle, ink: Ink::Text, opacity: 1.0, weight: 400 }
}

fn text(x: f64, y: f64, content: String, style: TextStyle) -> Shape {
    let TextStyle { anchor, baseline, ink, opacity, weight } = style;
    Shape::Text { x: round(x), y: round(y), text: content, anchor, baseline, size: TEXT_SIZE, weight, ink, opacity }
}

pub fn format_value(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        format!("{value:.1}").replace('.', ",")
    }
}

fn nice_max(value: f64) -> f64 {
    if value <= 0.0 {
        return 1.0;
    }
    let magnitude = 10f64.powf(value.log10().floor());
    let normalized = value / magnitude;
    let nice = match normalized {
        n if n <= 1.0 => 1.0,
        n if n <= 2.0 => 2.0,
        n if n <= 2.5 => 2.5,
        n if n <= 5.0 => 5.0,
        _ => 10.0,
    };
    nice * magnitude
}

fn slice_color(palette: &[String], index: usize, fallback: &str) -> String {
    if palette.is_empty() { fallback.to_string() } else { palette[index % palette.len()].clone() }
}

pub fn scene(data: &ChartData) -> ChartScene {
    let pie = matches!(data.kind, ChartKind::Pie | ChartKind::Donut);
    let mut shapes = if pie { pie_shapes(data) } else { cartesian_shapes(data) };
    if data.legend {
        shapes.extend(legend_shapes(data, pie));
    }
    ChartScene { width: WIDTH, height: HEIGHT + if data.legend { LEGEND_HEIGHT } else { 0.0 }, shapes }
}

fn cartesian_shapes(data: &ChartData) -> Vec<Shape> {
    let plot_width = WIDTH - MARGIN_LEFT - MARGIN_RIGHT;
    let plot_height = HEIGHT - MARGIN_TOP - MARGIN_BOTTOM;
    let count = data.labels.len().max(1) as f64;
    let value_at = |series: &Series, index: usize| series.values.get(index).copied().unwrap_or(0.0);
    let peak = if data.stacked && data.kind == ChartKind::Bar {
        (0..data.labels.len()).map(|index| data.series.iter().map(|series| value_at(series, index).max(0.0)).sum::<f64>()).fold(0.0, f64::max)
    } else {
        data.series.iter().flat_map(|series| series.values.iter().copied()).fold(0.0, f64::max)
    };
    let max = nice_max(peak);
    let y = |value: f64| MARGIN_TOP + plot_height - (value.max(0.0) / max) * plot_height;
    let band = plot_width / count;
    let x = |index: usize| MARGIN_LEFT + band * (index as f64 + 0.5);
    let mut shapes = Vec::new();
    for ratio in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let tick = ratio * max;
        shapes.push(Shape::Line { x1: MARGIN_LEFT, y1: round(y(tick)), x2: WIDTH - MARGIN_RIGHT, y2: round(y(tick)), width: 2.0, ink: Ink::Text, opacity: 0.12 });
        shapes.push(text(MARGIN_LEFT - 16.0, y(tick), format_value(tick), axis_style()));
    }
    for (index, label) in data.labels.iter().enumerate() {
        shapes.push(text(x(index), HEIGHT - MARGIN_BOTTOM + 40.0, label.clone(), category_style()));
    }
    if data.kind == ChartKind::Bar {
        let group = band * 0.72;
        let bar_width = if data.stacked { group } else { group / data.series.len().max(1) as f64 };
        for index in 0..data.labels.len() {
            let mut offset = 0.0;
            for (series_index, series) in data.series.iter().enumerate() {
                let value = value_at(series, index).max(0.0);
                let left = if data.stacked { x(index) - group / 2.0 } else { x(index) - group / 2.0 + series_index as f64 * bar_width };
                let top = if data.stacked { y(offset + value) } else { y(value) };
                let bottom = if data.stacked { y(offset) } else { y(0.0) };
                offset += value;
                shapes.push(Shape::Rect {
                    x: round(left + 3.0),
                    y: round(top),
                    w: round((bar_width - 6.0).max(2.0)),
                    h: round((bottom - top).max(0.0)),
                    radius: 8.0,
                    ink: color(&series.color),
                    opacity: 1.0,
                });
                if data.values {
                    shapes.push(text(left + bar_width / 2.0, top - 12.0, format_value(value_at(series, index)), value_style()));
                }
            }
        }
    } else {
        let baseline = round(y(0.0));
        for series in data.series {
            let points: Vec<[f64; 2]> = series.values.iter().enumerate().map(|(index, value)| [round(x(index)), round(y(*value))]).collect();
            if data.kind == ChartKind::Area && !points.is_empty() {
                let mut outline = vec![[points[0][0], baseline]];
                outline.extend(points.iter().copied());
                outline.push([points[points.len() - 1][0], baseline]);
                shapes.push(Shape::Polygon { points: outline, ink: color(&series.color), opacity: 0.18 });
            }
            shapes.push(Shape::Polyline { points: points.clone(), width: 6.0, ink: color(&series.color), opacity: 1.0 });
            for (index, point) in points.iter().enumerate() {
                shapes.push(Shape::Circle { cx: point[0], cy: point[1], r: 9.0, ink: color(&series.color), opacity: 1.0 });
                if data.values {
                    shapes.push(text(point[0], point[1] - 22.0, format_value(series.values[index]), value_style()));
                }
            }
        }
    }
    shapes
}

fn pie_shapes(data: &ChartData) -> Vec<Shape> {
    let Some(series) = data.series.first() else { return Vec::new() };
    let values: Vec<f64> = series.values.iter().map(|value| value.max(0.0)).collect();
    let sum: f64 = values.iter().sum();
    let total = if sum > 0.0 { sum } else { 1.0 };
    let (cx, cy) = (WIDTH / 2.0, HEIGHT / 2.0);
    let outer = HEIGHT / 2.0 - 24.0;
    let inner = if data.kind == ChartKind::Donut { outer * 0.58 } else { 0.0 };
    let label_radius = if inner > 0.0 { (outer + inner) / 2.0 } else { outer * 0.62 };
    let mut angle = -FRAC_PI_2;
    let mut shapes = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let start = angle;
        let sweep = value / total * TAU;
        angle += sweep;
        shapes.push(Shape::Sector {
            cx,
            cy,
            outer,
            inner: round(inner),
            start: start.to_degrees(),
            end: angle.to_degrees(),
            ink: color(&slice_color(data.palette, index, &series.color)),
            stroke: Ink::Background,
            stroke_width: 4.0,
        });
        if data.values && *value > 0.0 {
            let middle = start + sweep / 2.0;
            let percent = (value / total * 100.0).round();
            shapes.push(text(cx + middle.cos() * label_radius, cy + middle.sin() * label_radius, format!("{percent}%"), percent_style()));
        }
    }
    shapes
}

fn legend_shapes(data: &ChartData, pie: bool) -> Vec<Shape> {
    let entries: Vec<(String, String)> = if pie {
        data.labels.iter().enumerate().map(|(index, label)| (label.clone(), slice_color(data.palette, index, "#888888"))).collect()
    } else {
        data.series.iter().map(|series| (series.name.clone(), series.color.clone())).collect()
    };
    let item_width = (WIDTH / entries.len().max(1) as f64).min(260.0);
    let start = WIDTH / 2.0 - item_width * entries.len() as f64 / 2.0;
    let top = HEIGHT + 16.0;
    entries
        .into_iter()
        .enumerate()
        .flat_map(|(index, (label, swatch))| {
            let left = start + index as f64 * item_width;
            [
                Shape::Rect { x: round(left), y: top, w: 24.0, h: 24.0, radius: 6.0, ink: color(&swatch), opacity: 1.0 },
                text(left + 36.0, top + 13.0, label, legend_style()),
            ]
        })
        .collect()
}
