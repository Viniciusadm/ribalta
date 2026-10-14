use std::collections::BTreeMap;

use crate::ir::{
    Animation, BandSpec, LayoutDefaults, LayoutGaps, CodeSpec, DividerSpec, Enter, IconSpec, InlineCodeSpec, Length, LinkSpec, ListSpec, Motion, Pose, QuoteSpec, ShadowSpec, Shadows, TableSpec, ThemeElements, Timing,
    TokenKind, TokenStyle, Transition,
};

pub const THEMES: &[&str] = &["paper", "ink", "editorial"];

#[derive(Debug, Clone)]
pub struct FontSpec {
    pub family: String,
    pub weights: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct HeadingSpec {
    pub size: f32,
    pub weight: u16,
    pub font: &'static str,
    pub leading: f32,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: BTreeMap<String, String>,
    pub fonts: BTreeMap<String, FontSpec>,
    pub text_size: f32,
    pub leading: f32,
    pub headings: [HeadingSpec; 6],
    pub page_pad: [f32; 2],
    pub gap: f32,
    pub code_dark: bool,
    pub palette: Vec<&'static str>,
}

pub const BUILTIN_STYLES: &[(&str, &str)] = &[
    ("muted", "color=muted"),
    ("accent", "color=accent"),
    ("lead", "size=xl color=muted"),
    ("kicker", "size=sm weight=bold uppercase tracking=0.14 color=accent"),
    ("display", "font=display"),
    ("huge", "font=display size=6xl leading=0.95"),
    ("card", "bg=surface radius=lg pad=lg shadow=md"),
    ("dark", "bg=ink color=paper"),
    ("center", "align=center"),
];

fn font(family: &str, weights: &[u16]) -> FontSpec {
    FontSpec { family: family.to_string(), weights: weights.to_vec() }
}

fn heading(size: f32, weight: u16, font: &'static str, leading: f32) -> HeadingSpec {
    HeadingSpec { size, weight, font, leading }
}

fn colors(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(name, value)| (name.to_string(), value.to_string())).collect()
}

pub fn builtin(name: &str) -> Option<Theme> {
    let palette = vec!["accent", "blue", "green", "yellow", "violet", "coral"];
    let shared = [
        ("coral", "#f2603d"),
        ("blue", "#2f6fdb"),
        ("green", "#23976a"),
        ("yellow", "#f2b632"),
        ("violet", "#7c5cd6"),
        ("white", "#ffffff"),
        ("black", "#000000"),
        ("ink", "#151821"),
        ("paper", "#fbf7ef"),
        ("transparent", "#00000000"),
    ];
    let theme = match name {
        "paper" => {
            let mut map = colors(&shared);
            map.extend(colors(&[
                ("bg", "#fbf7ef"),
                ("text", "#1f2430"),
                ("muted", "#6b6f7b"),
                ("accent", "#f2603d"),
                ("surface", "#ffffff"),
                ("line", "#1f243026"),
            ]));
            Theme {
                name: name.into(),
                colors: map,
                fonts: [
                    ("body".to_string(), font("Barlow", &[300, 400, 500, 600, 700, 900])),
                    ("display".to_string(), font("Archivo Black", &[])),
                    ("mono".to_string(), font("JetBrains Mono", &[400, 700])),
                ]
                .into(),
                text_size: 40.0,
                leading: 1.35,
                headings: [
                    heading(88.0, 400, "display", 1.02),
                    heading(64.0, 400, "display", 1.05),
                    heading(52.0, 700, "body", 1.1),
                    heading(40.0, 700, "body", 1.2),
                    heading(32.0, 700, "body", 1.2),
                    heading(26.0, 700, "body", 1.2),
                ],
                page_pad: [96.0, 128.0],
                gap: 32.0,
                code_dark: true,
                palette,
            }
        }
        "ink" => {
            let mut map = colors(&shared);
            map.extend(colors(&[
                ("bg", "#12141c"),
                ("text", "#f4f1ea"),
                ("muted", "#a3a7b3"),
                ("accent", "#ffb547"),
                ("surface", "#1d2130"),
                ("line", "#ffffff26"),
                ("ink", "#0b0c11"),
            ]));
            Theme {
                name: name.into(),
                colors: map,
                fonts: [
                    ("body".to_string(), font("Inter", &[300, 400, 500, 600, 700, 800, 900])),
                    ("display".to_string(), font("Inter", &[300, 400, 500, 600, 700, 800, 900])),
                    ("mono".to_string(), font("JetBrains Mono", &[400, 700])),
                ]
                .into(),
                text_size: 38.0,
                leading: 1.4,
                headings: [
                    heading(92.0, 800, "display", 1.0),
                    heading(66.0, 800, "display", 1.05),
                    heading(50.0, 700, "body", 1.1),
                    heading(40.0, 700, "body", 1.2),
                    heading(32.0, 600, "body", 1.2),
                    heading(26.0, 600, "body", 1.2),
                ],
                page_pad: [96.0, 128.0],
                gap: 32.0,
                code_dark: true,
                palette,
            }
        }
        "editorial" => {
            let mut map = colors(&shared);
            map.extend(colors(&[
                ("bg", "#f6f1e7"),
                ("text", "#1b1b1b"),
                ("muted", "#6d665c"),
                ("accent", "#b23a2b"),
                ("surface", "#fffdf8"),
                ("line", "#1b1b1b22"),
            ]));
            Theme {
                name: name.into(),
                colors: map,
                fonts: [
                    ("body".to_string(), font("Source Sans 3", &[300, 400, 600, 700, 900])),
                    ("display".to_string(), font("Fraunces", &[400, 600, 700, 900])),
                    ("mono".to_string(), font("JetBrains Mono", &[400, 700])),
                ]
                .into(),
                text_size: 40.0,
                leading: 1.4,
                headings: [
                    heading(96.0, 600, "display", 1.0),
                    heading(68.0, 600, "display", 1.05),
                    heading(52.0, 600, "display", 1.1),
                    heading(40.0, 700, "body", 1.2),
                    heading(32.0, 700, "body", 1.2),
                    heading(26.0, 700, "body", 1.2),
                ],
                page_pad: [104.0, 140.0],
                gap: 32.0,
                code_dark: false,
                palette,
            }
        }
        _ => return None,
    };
    Some(theme)
}

const EASE: [f64; 4] = [0.25, 0.1, 0.25, 1.0];

fn faded(dx: Option<Length>, dy: Option<Length>, scale: Option<f32>) -> Pose {
    Pose { opacity: 0.0, dx, dy, scale }
}

fn animation(duration: u32, easing: [f64; 4], from: Pose) -> Animation {
    Animation { timing: Timing { duration, easing }, from }
}

pub fn standard_motion() -> Motion {
    let px = |value: f32| Some(Length::Px { value });
    let enter = |from: Pose| animation(360, EASE, from);
    Motion {
        enter: BTreeMap::from([
            (Enter::Fade, enter(faded(None, None, None))),
            (Enter::Up, enter(faded(None, px(16.0), None))),
            (Enter::Down, enter(faded(None, px(-16.0), None))),
            (Enter::Left, enter(faded(px(32.0), None, None))),
            (Enter::Right, enter(faded(px(-32.0), None, None))),
            (Enter::Zoom, enter(faded(None, None, Some(0.92)))),
        ]),
        transitions: BTreeMap::from([
            (Transition::Fade, animation(420, EASE, faded(None, None, None))),
            (Transition::Slide, animation(480, [0.2, 0.7, 0.2, 1.0], faded(Some(Length::Pct { value: 6.0 }), None, None))),
            (Transition::Zoom, animation(420, EASE, faded(None, None, Some(0.92)))),
        ]),
        reveal: Timing { duration: 420, easing: EASE },
        variant: Timing { duration: 300, easing: EASE },
    }
}

fn shadow(y: f32, blur: f32, alpha: &str) -> ShadowSpec {
    ShadowSpec { x: 0.0, y, blur, color: format!("#0f121e{alpha}") }
}

pub fn standard_elements(theme: &Theme) -> ThemeElements {
    let color = |token: &str, fallback: &str| theme.colors.get(token).cloned().unwrap_or_else(|| fallback.to_string());
    let accent = color("accent", "#f2603d");
    let line = color("line", "#00000033");
    let mono = theme.fonts.get("mono").map_or_else(|| "JetBrains Mono".to_string(), |spec| spec.family.clone());
    let (code_bg, code_color) = if theme.code_dark { ("#1e1e1e", "#d4d4d4") } else { ("#f6f8fa", "#24292f") };
    let syntax = syntax_palette(if theme.code_dark { DARK_SYNTAX } else { LIGHT_SYNTAX });
    ThemeElements {
        shadows: Shadows { sm: shadow(2.0, 6.0, "1f"), md: shadow(10.0, 30.0, "29"), lg: shadow(24.0, 60.0, "3d") },
        list: ListSpec { gap_em: 0.35, indent_em: 1.3, marker_color: accent.clone(), marker_weight: 700, nested_gap_em: 0.3, nested_scale: 0.85 },
        quote: QuoteSpec { gap_em: 0.5, indent: 40.0, bar_width: 8.0, bar_color: accent.clone(), italic: true },
        code: CodeSpec { font: mono.clone(), size: 28.0, leading: 1.5, pad: [32.0, 40.0], radius: 20.0, bg: code_bg.into(), color: code_color.into(), syntax },
        inline_code: InlineCodeSpec { font: mono, scale: 0.85, pad_em: [0.05, 0.3], radius_em: 0.25, tint: 0.1 },
        link: LinkSpec { color: accent, underline_em: 0.06, underline_offset_em: 0.15 },
        table: TableSpec { size: 30.0, cell_pad: [14.0, 28.0], rule_width: 2.0, header_rule_width: 4.0, rule_color: line.clone(), header_weight: 700 },
        divider: DividerSpec { thickness: 4.0, radius: 2.0, color: line },
        icon: IconSpec { size: 96.0, stroke: 1.75 },
    }
}

const DARK_SYNTAX: &[(TokenKind, &str, bool)] = &[
    (TokenKind::Plain, "#d4d4d4", false),
    (TokenKind::Comment, "#6a9955", true),
    (TokenKind::Keyword, "#569cd6", false),
    (TokenKind::String, "#ce9178", false),
    (TokenKind::Number, "#b5cea8", false),
    (TokenKind::Constant, "#4fc1ff", false),
    (TokenKind::Function, "#dcdcaa", false),
    (TokenKind::Type, "#4ec9b0", false),
    (TokenKind::Operator, "#d4d4d4", false),
    (TokenKind::Punctuation, "#d4d4d4", false),
    (TokenKind::Tag, "#569cd6", false),
    (TokenKind::Attribute, "#9cdcfe", false),
    (TokenKind::Property, "#9cdcfe", false),
    (TokenKind::Variable, "#9cdcfe", false),
];

const LIGHT_SYNTAX: &[(TokenKind, &str, bool)] = &[
    (TokenKind::Plain, "#24292f", false),
    (TokenKind::Comment, "#6e7781", true),
    (TokenKind::Keyword, "#cf222e", false),
    (TokenKind::String, "#0a3069", false),
    (TokenKind::Number, "#0550ae", false),
    (TokenKind::Constant, "#0550ae", false),
    (TokenKind::Function, "#8250df", false),
    (TokenKind::Type, "#953800", false),
    (TokenKind::Operator, "#24292f", false),
    (TokenKind::Punctuation, "#24292f", false),
    (TokenKind::Tag, "#116329", false),
    (TokenKind::Attribute, "#0550ae", false),
    (TokenKind::Property, "#0550ae", false),
    (TokenKind::Variable, "#953800", false),
];

fn syntax_palette(entries: &[(TokenKind, &str, bool)]) -> BTreeMap<TokenKind, TokenStyle> {
    entries.iter().map(|(kind, color, italic)| (*kind, TokenStyle { color: (*color).to_string(), italic: *italic })).collect()
}

pub fn standard_layout() -> LayoutDefaults {
    LayoutDefaults {
        gaps: LayoutGaps { stack: 32.0, row: 16.0, columns: 32.0, grid: 32.0, boxed: 20.0 },
        grid_columns: 2,
        band: BandSpec { gap: 16.0, edge_scale: 0.4, height_scale: 0.6 },
        chart_max_height: 560.0,
    }
}
