use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::attrs::{Attrs, Pair, matching_brace, parse_attrs};
use crate::block::{Directive, List, Node, Table, parse_blocks};
use crate::diagnostic::Diagnostics;
use crate::document::{Assignment, ComponentDef, DataDef, Preamble, RegionDef, PageSource, assignment, separator_attrs};
use crate::eval::{self, Binding, Folded, MAX_ITEMS, Scope};
use crate::expr::{self, Ast, AstKind, ExprError, RawAction, path_at};
use crate::inline::{RawInline, parse_inline};
use crate::ir::{Field, 
    Action, Behavior, BinaryOp, Block, BlockKind, ChartKind, Expr, FontFamily, HAlign, ImageFit, ImageSource, Inline, LayoutKind, ListItem, Mode, PageSize, Paint, Scalar, Scorm, Series, Page, SourceRange, Style,
    ThemeIr, Transition, VAlign, Variant,
};
use crate::chart::{self, ChartData};
use crate::highlight::highlight;
use crate::style::{self, Invalid, Parsed};
use crate::text::{tr, Line, Span, Text, suggest};
use crate::theme::{BUILTIN_STYLES, FontSpec, THEMES, Theme, builtin, standard_elements, standard_layout, standard_motion};
use crate::value::{Record, Type, Value, format_number, parse_number};
use crate::icons;
use crate::{LANGUAGE_VERSION, Profile};

pub const REGION_FIELDS: &[&str] = &["page", "pages", "title", "author", "date"];

const DATE_MARK: char = '\u{E000}';
const DYNAMIC_MARK: char = '\u{E001}';

pub const BUILTIN_BLOCKS: &[&str] = &["stack", "row", "columns", "grid", "box", "steps", "notes", "chart", "table", "icon", "spacer", "divider", "input", "sound", "slot", "if", "else", "for"];

const STYLE_KEYS: &[&str] = &[
    "color", "bg", "image", "font", "size", "weight", "italic", "uppercase", "underline", "align", "valign", "pad", "gap", "radius", "border",
    "shadow", "w", "h", "x", "y", "opacity", "leading", "tracking", "shrink", "enter", "step", "if", "click", "key",
];

const EXPRESSION_KEYS: &[&str] = &["if", "click", "data", "on-enter", "on-timeout", "bind", "on-submit"];

const MIN_PAGE: u32 = 16;
const MAX_PAGE: u32 = 16384;

fn page_length(text: &str) -> Option<u32> {
    text.trim().parse::<u32>().ok().filter(|value| (MIN_PAGE..=MAX_PAGE).contains(value))
}

fn page_height(text: &str) -> Option<Option<u32>> {
    if text.trim() == "auto" { Some(None) } else { page_length(text).map(Some) }
}

pub fn page_size(text: &str, base: PageSize) -> Option<PageSize> {
    let text = text.trim();
    if !text.contains('=') {
        let (width, height) = text.split_once('x')?;
        return Some(PageSize { width: page_length(width)?, height: page_height(height)? });
    }
    let mut size = base;
    for part in text.split_whitespace() {
        match part.split_once('=')? {
            ("width", value) => size.width = page_length(value)?,
            ("height", value) => size.height = page_height(value)?,
            _ => return None,
        }
    }
    Some(size)
}

pub fn attribute_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();
    for name in STYLE_KEYS.iter().chain(Kind::Any.extra_keys()).chain(Kind::Page.extra_keys()) {
        if !names.contains(name) {
            names.push(name);
        }
    }
    names
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Page,
    Text,
    Image,
    Layout(LayoutKind),
    Chart,
    Table,
    Input,
    Sound,
    Span,
    Any,
}

impl Kind {
    fn extra_keys(self) -> &'static [&'static str] {
        match self {
            Kind::Page => &["header", "footer", "overlay", "cover", "hidden", "next", "steps", "transition", "on-enter", "nav", "timeout", "on-timeout", "width", "height"],
            Kind::Image => &["fit"],
            Kind::Layout(LayoutKind::Columns) => &["widths"],
            Kind::Layout(LayoutKind::Grid) => &["cols"],
            Kind::Layout(LayoutKind::Row) => &["wrap"],
            Kind::Chart => &["type", "legend", "stacked", "values", "colors", "data", "label", "series"],
            Kind::Table => &["data", "columns"],
            Kind::Input => &["bind", "placeholder", "on-submit"],
            Kind::Sound => &["name", "autoplay", "loop", "volume"],
            Kind::Any => &[
                "fit", "widths", "cols", "wrap", "type", "legend", "stacked", "values", "colors", "header", "footer", "overlay", "cover", "hidden", "next", "steps",
                "transition", "on-enter", "nav", "timeout", "on-timeout", "width", "height", "data", "label", "series", "columns", "bind", "placeholder", "on-submit", "name", "autoplay", "loop", "volume",
            ],
            _ => &[],
        }
    }
}

#[derive(Default)]
struct Resolved {
    style: Style,
    step: Option<u32>,
    id: Option<Text>,
    extra: Vec<Pair>,
    behavior: Behavior,
    key_span: Option<Span>,
    dropped: bool,
}

impl Resolved {
    fn extra(&self, key: &str) -> Option<&Pair> {
        self.extra.iter().rev().find(|pair| pair.key.value == key)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Region {
    Header,
    Footer,
    Overlay,
}

impl Region {
    fn name(self) -> &'static str {
        match self {
            Region::Header => "header",
            Region::Footer => "footer",
            Region::Overlay => "overlay",
        }
    }
}

#[derive(Clone)]
struct RegionSource {
    def: RegionDef,
    from_prelude: bool,
    reported: bool,
}

#[derive(Clone, PartialEq, Eq)]
enum Barrier {
    Component,
    Region,
}

#[derive(Default)]
struct Frame {
    names: HashMap<String, Binding>,
    barrier: Option<Barrier>,
    missing: BTreeSet<String>,
    content: Vec<Block>,
    slots: HashMap<String, Vec<Block>>,
    slot_names: BTreeSet<String>,
    used_content: bool,
}

enum Chain {
    Done,
    Open(Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Declaration {
    Let,
    State,
    Extern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExternUse {
    Change,
    Persist,
    Score,
}

pub struct Lowerer<'d> {
    diags: &'d mut Diagnostics,
    theme: Theme,
    styles: HashMap<String, Attrs>,
    components: HashMap<String, ComponentDef>,
    prelude_components: HashSet<String>,
    expansion: Vec<String>,
    tracking: bool,
    step: u32,
    max_step: u32,
    notes: Vec<String>,
    page_ids: HashSet<String>,
    current_page: usize,
    anchors: Vec<(String, usize, Span)>,
    header: Option<RegionSource>,
    footer: Option<RegionSource>,
    overlay: Option<RegionSource>,
    title: String,
    author: String,
    frames: Vec<Frame>,
    dynamic: Vec<Expr>,
    expr_origin: Span,
    iterations: usize,
    document_state: BTreeMap<String, Scalar>,
    record_fields: HashMap<String, BTreeMap<String, Type>>,
    empty_data: HashSet<String>,
    defined: HashSet<String>,
    assigned: HashMap<String, Option<BTreeSet<String>>>,
    comparisons: Vec<(String, String, Span)>,
    targets: Vec<(String, Span, bool)>,
    hidden: Vec<(String, Span)>,
    pub profile: Profile,
    transition: Option<Transition>,
    pub mode: Mode,
    in_action: bool,
    pub uses_random: bool,
    uses_elapsed: bool,
    sounds: HashMap<String, Span>,
    sound_refs: Vec<(String, Span)>,
    pub persist: Vec<String>,
    pub externs: Vec<String>,
    pub scorm: Option<Scorm>,
    interpolation_failed: bool,
    pub total_pages: usize,
    pub page: PageSize,
}

impl Scope for Lowerer<'_> {
    fn binding(&mut self, name: &str) -> Option<Binding> {
        self.lookup(name)
    }

    fn names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for frame in self.visible_frames() {
            names.extend(frame.names.keys().cloned());
        }
        names
    }

    fn note_step(&mut self, step: u32) {
        if self.tracking {
            self.max_step = self.max_step.max(step);
        }
    }

    fn allows_random(&self) -> bool {
        self.in_action
    }

    fn allows_items(&self) -> bool {
        !self.in_action
    }

    fn note_random(&mut self) {
        self.uses_random = true;
    }

    fn note_elapsed(&mut self) {
        self.uses_elapsed = true;
    }

    fn note_comparison(&mut self, state: &str, value: &str, from: usize, to: usize) {
        let origin = self.expr_origin;
        self.comparisons.push((state.to_string(), value.to_string(), Span::new(origin.line, origin.from + from, origin.from + to)));
    }
}

impl<'d> Lowerer<'d> {
    pub fn new(page: PageSize, prelude: Option<&Preamble>, preamble: &Preamble, diags: &'d mut Diagnostics) -> Self {
        let theme_name = preamble.theme.as_ref().or(prelude.and_then(|prelude| prelude.theme.as_ref()));
        let reported = preamble.theme.is_some();
        let theme = match theme_name {
            Some(name) => builtin(&name.value).unwrap_or_else(|| {
                if reported {
                    let pending = diags.error("R513", name.span, tr(format!("tema desconhecido: `{}`", name.value), format!("unknown theme: `{}`", name.value)));
                    match suggest(&name.value, THEMES.iter().copied()) {
                        Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                        None => pending.hint(tr(format!("`@theme` escolhe um tema base: {}. Para criar o seu, mantenha uma base e ajuste com @color, @font e @style.", THEMES.join(", ")), format!("`@theme` picks a base theme: {}. To make your own, keep a base and adjust it with @color, @font and @style.", THEMES.join(", ")))),
                    };
                }
                builtin("paper").expect("paper theme exists")
            }),
            None => builtin("paper").expect("paper theme exists"),
        };
        let mut globals = Frame::default();
        globals.names.insert("step".to_string(), Binding::Step);
        globals.names.insert("elapsed".to_string(), Binding::Elapsed);
        let mut lowerer = Self {
            diags,
            theme,
            styles: HashMap::new(),
            components: HashMap::new(),
            prelude_components: HashSet::new(),
            expansion: Vec::new(),
            tracking: false,
            step: 0,
            max_step: 0,
            notes: Vec::new(),
            page_ids: HashSet::new(),
            current_page: 0,
            anchors: Vec::new(),
            header: None,
            footer: None,
            overlay: None,
            title: preamble.title.clone().unwrap_or_default(),
            author: preamble.author.clone().unwrap_or_default(),
            frames: vec![globals],
            dynamic: Vec::new(),
            expr_origin: Span::default(),
            iterations: 0,
            document_state: BTreeMap::new(),
            record_fields: HashMap::new(),
            empty_data: HashSet::new(),
            defined: HashSet::new(),
            assigned: HashMap::new(),
            comparisons: Vec::new(),
            targets: Vec::new(),
            hidden: Vec::new(),
            profile: Profile::Default,
            transition: None,
            mode: Mode::Default,
            in_action: false,
            uses_random: false,
            uses_elapsed: false,
            sounds: HashMap::new(),
            sound_refs: Vec::new(),
            persist: Vec::new(),
            externs: Vec::new(),
            scorm: None,
            interpolation_failed: false,
            total_pages: 0,
            page,
        };
        let mut builtin_diags = Diagnostics::default();
        for (name, source) in BUILTIN_STYLES {
            lowerer.styles.insert((*name).to_string(), parse_attrs(source, 0, 0, &mut builtin_diags));
        }
        if let Some(prelude) = prelude {
            let reported = std::mem::take(lowerer.diags);
            lowerer.load(prelude);
            *lowerer.diags = reported;
            lowerer.prelude_components = lowerer.components.keys().cloned().collect();
            lowerer.defined.clear();
            lowerer.header = prelude.header.clone().map(|def| RegionSource { def, from_prelude: true, reported: true });
            lowerer.footer = prelude.footer.clone().map(|def| RegionSource { def, from_prelude: true, reported: true });
            lowerer.overlay = prelude.overlay.clone().map(|def| RegionSource { def, from_prelude: true, reported: true });
        }
        lowerer.load(preamble);
        if let Some(def) = &preamble.header {
            lowerer.header = Some(RegionSource { def: def.clone(), from_prelude: false, reported: false });
        }
        if let Some(def) = &preamble.footer {
            lowerer.footer = Some(RegionSource { def: def.clone(), from_prelude: false, reported: false });
        }
        if let Some(def) = &preamble.overlay {
            lowerer.overlay = Some(RegionSource { def: def.clone(), from_prelude: false, reported: false });
        }
        lowerer
    }

    fn load(&mut self, preamble: &Preamble) {
        if let Some(version) = &preamble.version {
            match version.value.parse::<u32>() {
                Ok(n) if (1..=LANGUAGE_VERSION).contains(&n) => {}
                _ => {
                    self.diags
                        .error("R514", version.span, tr(format!("versão da linguagem não suportada: `{}`", version.value), format!("unsupported language version: `{}`", version.value)))
                        .hint(tr(format!("A versão atual é {LANGUAGE_VERSION}"), format!("The current version is {LANGUAGE_VERSION}")));
                }
            }
        }
        if let Some(text) = &preamble.page {
            match page_size(&text.value, self.page) {
                Some(page) => self.page = page,
                None => {
                    self.diags
                        .error("R515", text.span, tr(format!("tamanho de página inválido: `{}`", text.value), format!("invalid page size: `{}`", text.value)))
                        .hint(tr("Use largura x altura, como @page 1920x1080, ou @page width=800 height=auto", "Use width x height, such as @page 1920x1080, or @page width=800 height=auto"));
                }
            }
        }
        if let Some(transition) = &preamble.transition {
            match style::transition(&transition.value) {
                Ok(value) => self.transition = Some(value),
                Err(invalid) => self.report(transition.span, invalid),
            }
        }
        if let Some(mode) = &preamble.mode {
            match mode.value.as_str() {
                "default" => self.mode = Mode::Default,
                "game" => self.mode = Mode::Game,
                other => {
                    self.diags.error("R534", mode.span, tr(format!("modo desconhecido: `{other}`"), format!("unknown mode: `{other}`"))).hint(tr("Use @mode default (o padrão) ou @mode game", "Use @mode default (the default) or @mode game"));
                }
            }
        }
        for data in &preamble.data {
            self.data(data);
        }
        let mut ordered: Vec<(Declaration, &Assignment)> = preamble
            .lets
            .iter()
            .map(|assignment| (Declaration::Let, assignment))
            .chain(preamble.states.iter().map(|assignment| (Declaration::State, assignment)))
            .chain(preamble.externs.iter().map(|assignment| (Declaration::Extern, assignment)))
            .collect();
        ordered.sort_by_key(|(_, assignment)| (assignment.name.span.line, assignment.name.span.from));
        for (declaration, assignment) in ordered {
            match declaration {
                Declaration::Let => self.define_let(assignment, 0),
                Declaration::State => {
                    if let Some((name, value)) = self.define_state(assignment, 0, false) {
                        self.document_state.insert(name, value);
                    }
                }
                Declaration::Extern => {
                    if let Some((name, value)) = self.define_state(assignment, 0, true) {
                        self.assigned.insert(name.clone(), None);
                        self.externs.push(name.clone());
                        self.document_state.insert(name, value);
                    }
                }
            }
        }
        for name in &preamble.persist {
            let span = Span::new(name.span.line, name.span.from, name.span.from + name.value.len());
            match self.frames[0].names.get(&name.value) {
                Some(Binding::State(_)) if self.externs.contains(&name.value) => {
                    self.report_extern(&name.value, span, ExternUse::Persist);
                }
                Some(Binding::State(_)) => {
                    if !self.persist.contains(&name.value) {
                        self.persist.push(name.value.clone());
                    }
                }
                Some(_) => {
                    self.diags
                        .error("R617", span, tr(format!("`@persist` só guarda variáveis de @state, e `${}` não é uma", name.value), format!("`@persist` only keeps @state variables, and `${}` is not one", name.value)))
                        .hint(tr("Declare a variável com @state no início do documento", "Declare the variable with @state at the start of the document"));
                }
                None => {
                    let names: Vec<String> = self.frames[0].names.iter().filter(|(_, binding)| matches!(binding, Binding::State(_))).map(|(name, _)| name.clone()).collect();
                    let pending = self.diags.error("R601", span, tr(format!("variável desconhecida: `${}`", name.value), format!("unknown variable: `${}`", name.value)));
                    match suggest(&name.value, names.iter().map(String::as_str)) {
                        Some(found) => pending.hint(tr(format!("Você quis dizer `${found}`?"), format!("Did you mean `${found}`?"))),
                        None => pending.hint(tr(format!("Declare-a antes: @state {} = ...", name.value), format!("Declare it first: @state {} = ...", name.value))),
                    };
                }
            }
        }
        if let Some(scorm) = &preamble.scorm {
            self.scorm = self.scorm(scorm);
        }
        for (token, value) in &preamble.colors {
            if !valid_token(&token.value) {
                self.diags.error("R516", token.span, tr(format!("nome de cor inválido: `{}`", token.value), format!("invalid color name: `{}`", token.value))).hint(tr("Use letras minúsculas, números e hífens", "Use lowercase letters, numbers and hyphens"));
                continue;
            }
            let value = Text { value: self.interpolate(value, false).0, span: value.span };
            match style::color(&value.value, &self.theme) {
                Ok(color) => {
                    self.theme.colors.insert(token.value.clone(), color);
                }
                Err(invalid) => self.report(value.span, invalid),
            }
        }
        for font in &preamble.fonts {
            if !valid_token(&font.name.value) {
                self.diags.error("R517", font.name.span, tr(format!("nome de fonte inválido: `{}`", font.name.value), format!("invalid font name: `{}`", font.name.value))).hint(tr("Use letras minúsculas, números e hífens", "Use lowercase letters, numbers and hyphens"));
                continue;
            }
            self.theme.fonts.insert(font.name.value.clone(), FontSpec { family: font.family.value.clone(), weights: font.weights.clone() });
        }
        for (name, attrs) in &preamble.styles {
            if !valid_token(&name.value) {
                self.diags.error("R518", name.span, tr(format!("nome de estilo inválido: `{}`", name.value), format!("invalid style name: `{}`", name.value))).hint(tr("Use letras minúsculas, números e hífens", "Use lowercase letters, numbers and hyphens"));
                continue;
            }
            self.resolve(Some(attrs), Kind::Any);
            self.styles.insert(name.value.clone(), attrs.clone());
        }
        for component in &preamble.components {
            let name = &component.name.value;
            for (param, _) in &component.params {
                if matches!(param.value.as_str(), "step" | "if" | "click" | "key") {
                    self.diags
                        .error("R419", param.span, tr(format!("`{}` não pode ser parâmetro: ele já é um atributo de quem usa o componente", param.value), format!("`{}` cannot be a parameter: it is already an attribute of the component's usage", param.value)))
                        .hint(tr(format!("Escolha outro nome, como {}-texto; no uso, `{}=…` continua funcionando", param.value, param.value), format!("Pick another name, such as {}-text; on usage, `{}=…` keeps working", param.value, param.value)));
                }
            }
            if BUILTIN_BLOCKS.contains(&name.as_str()) {
                self.diags.error("R412", component.name.span, tr(format!("`{name}` já é um bloco da linguagem e não pode ser redefinido"), format!("`{name}` is already a language block and cannot be redefined")));
            } else if self.components.contains_key(name) && !self.prelude_components.remove(name) {
                self.diags.error("R413", component.name.span, tr(format!("o componente `{name}` foi definido mais de uma vez"), format!("component `{name}` was defined more than once")));
            } else {
                self.components.insert(name.clone(), component.clone());
            }
        }
    }

    fn declare(&mut self, name: &Text, frame: usize) -> bool {
        let valid = name.value.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && name.value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
            && !name.value.ends_with('-');
        if !valid {
            self.diags
                .error("R609", name.span, tr(format!("nome de variável inválido: `{}`", name.value), format!("invalid variable name: `{}`", name.value)))
                .hint(tr("Use letras minúsculas, números, hífens e sublinhados, começando por uma letra", "Use lowercase letters, numbers, hyphens and underscores, starting with a letter"));
            return false;
        }
        if name.value == "step" {
            self.diags.error("R610", name.span, tr("`step` é reservado: `$step` é a etapa atual da página", "`step` is reserved: `$step` is the current step of the page"));
            return false;
        }
        if name.value == "elapsed" {
            self.diags.error("R610", name.span, tr("`elapsed` é reservado: `$elapsed` são os segundos desde que a página abriu", "`elapsed` is reserved: `$elapsed` is the seconds since the page opened"));
            return false;
        }
        let taken = if frame == 0 { !self.defined.insert(name.value.clone()) } else { self.frames[frame].names.contains_key(&name.value) || self.defined.contains(&name.value) };
        if taken {
            self.diags.error("R610", name.span, tr(format!("`${}` já foi definido", name.value), format!("`${}` was already defined", name.value))).hint(tr("Cada nome de @let, @data e @state precisa ser único", "Each @let, @data and @state name must be unique"));
            return false;
        }
        true
    }

    fn data(&mut self, data: &DataDef) {
        if !self.declare(&data.name, 0) {
            return;
        }
        let Some(table) = &data.table else {
            return;
        };
        let mut keys = Vec::new();
        for cell in &table.header {
            let valid = path_at(&cell.value, 0).is_some_and(|(path, end)| path.len() == 1 && end == cell.value.len());
            if !valid {
                self.diags
                    .error("R609", cell.span, tr(format!("nome de campo inválido: `{}`", cell.value), format!("invalid field name: `{}`", cell.value)))
                    .hint(tr("Use letras, números, hífens e sublinhados, começando por uma letra", "Use letters, numbers, hyphens and underscores, starting with a letter"));
            }
            keys.push(cell.value.clone());
        }
        let numeric: Vec<bool> = (0..keys.len())
            .map(|column| {
                let cells: Vec<&Text> = table.rows.iter().filter_map(|row| row.get(column)).filter(|cell| !cell.value.is_empty()).collect();
                !cells.is_empty() && cells.iter().all(|cell| parse_number(&cell.value).is_some())
            })
            .collect();
        let mut rows = Vec::new();
        for row in &table.rows {
            let mut record = Record::default();
            for (column, key) in keys.iter().enumerate() {
                let raw = row.get(column).map_or("", |cell| cell.value.as_str());
                let value = if numeric[column] { Value::Number(parse_number(raw).unwrap_or(0.0)) } else { Value::from_cell(raw) };
                record.insert(key.clone(), value);
            }
            rows.push(Value::Record(record));
        }
        if rows.is_empty() {
            self.empty_data.insert(data.name.value.clone());
        }
        self.frames[0].names.insert(data.name.value.clone(), Binding::Value(Value::List(rows)));
    }

    fn define_let(&mut self, assignment: &Assignment, frame: usize) {
        if !self.declare(&assignment.name, frame) {
            return;
        }
        match self.expression(&assignment.value) {
            Some(Folded::Static(value)) => {
                self.frames[frame].names.insert(assignment.name.value.clone(), Binding::Value(value));
            }
            Some(Folded::Dynamic(..)) => {
                self.diags
                    .error("R604", assignment.value.span, tr("`@let` não pode depender do estado do documento", "`@let` cannot depend on the document state"))
                    .hint(tr("Use @state para valores que mudam durante o documento", "Use @state for values that change while the document runs"));
            }
            None => {}
        }
    }

    fn define_state(&mut self, assignment: &Assignment, frame: usize, external: bool) -> Option<(String, Scalar)> {
        if !self.declare(&assignment.name, frame) {
            return None;
        }
        let example = assignment.value.value.trim().strip_prefix('$').unwrap_or_default();
        if external && self.empty_data.contains(example) {
            self.diags
                .error("R602", assignment.value.span, tr(format!("a tabela `${example}` precisa de pelo menos uma linha de exemplo"), format!("the table `${example}` needs at least one example row")))
                .hint(tr("A primeira linha define os tipos dos campos e é o que o editor mostra", "The first row sets the field types and is what the editor shows"));
            return None;
        }
        let value = match self.expression(&assignment.value)? {
            Folded::Static(value) => value,
            Folded::Dynamic(..) => {
                self.diags.error("R604", assignment.value.span, tr("o valor inicial de `@state` precisa ser fixo", "the initial value of `@state` must be fixed"));
                return None;
            }
        };
        let scalar = match &value {
            Value::List(items) if items.first().is_some_and(|item| matches!(item, Value::Record(_))) => {
                if !external {
                    self.diags
                        .error("R602", assignment.value.span, tr("listas de registros só valem em @extern", "lists of records are only allowed in @extern"))
                        .hint(tr("Quem exibe o documento fornece a lista; declare-a com @extern nome = $tabela", "The host that shows the document provides the list; declare it with @extern name = $table"));
                    return None;
                }
                let (rows, fields) = self.record_rows(items, assignment.value.span)?;
                self.record_fields.insert(assignment.name.value.clone(), fields);
                let name = assignment.name.value.clone();
                self.frames[frame].names.insert(name.clone(), Binding::State(Type::Records));
                return Some((name, Scalar::Records(rows)));
            }
            Value::List(items) => {
                let mut texts: Vec<String> = Vec::new();
                for item in items {
                    let Value::Text(text) = item else {
                        self.diags
                            .error("R602", assignment.value.span, tr(format!("listas do `@state` guardam textos, não {}", item.ty().label()), format!("`@state` lists hold text, not {}", item.ty().label())))
                            .hint(tr("Exemplo: @state mochila = [chave, mapa]", "Example: @state backpack = [key, map]"));
                        return None;
                    };
                    if !texts.contains(text) {
                        texts.push(text.clone());
                    }
                }
                Scalar::List(texts)
            }
            other => match other.scalar() {
                Some(scalar) => scalar,
                None => {
                    self.diags.error(
                        "R602",
                        assignment.value.span,
                        tr(format!("`@state` guarda verdadeiro/falso, número, texto ou lista de textos, não {}", value.ty().label()), format!("`@state` holds true/false, a number, text or a list of text, not {}", value.ty().label())),
                    );
                    return None;
                }
            },
        };
        let name = assignment.name.value.clone();
        match &scalar {
            Scalar::Text(text) => {
                self.assigned.insert(name.clone(), Some(BTreeSet::from([text.clone()])));
            }
            Scalar::List(items) => {
                self.assigned.insert(name.clone(), Some(items.iter().cloned().collect()));
            }
            _ => {}
        }
        self.frames[frame].names.insert(name.clone(), Binding::State(value.ty()));
        Some((name, scalar))
    }

    fn record_rows(&mut self, items: &[Value], span: Span) -> Option<(Vec<BTreeMap<String, Field>>, BTreeMap<String, Type>)> {
        let mut fields: BTreeMap<String, Type> = BTreeMap::new();
        let mut rows = Vec::new();
        for item in items {
            let Value::Record(record) = item else {
                self.diags.error("R602", span, tr("a lista mistura registros com outros valores", "the list mixes records with other values"));
                return None;
            };
            let mut row = BTreeMap::new();
            for key in record.keys() {
                let value = record.get(key).cloned().unwrap_or(Value::Text(String::new()));
                let field = match &value {
                    Value::Bool(flag) => Field::Bool(*flag),
                    Value::Number(number) => Field::Number(*number),
                    Value::Text(text) => Field::Text(text.clone()),
                    other => {
                        self.diags.error("R602", span, tr(format!("o campo `{key}` guarda {}, mas campos guardam texto, número ou verdadeiro/falso", other.ty().label()), format!("the field `{key}` holds {}, but fields hold text, numbers or true/false", other.ty().label())));
                        return None;
                    }
                };
                match fields.get(key) {
                    Some(known) if *known != value.ty() => {
                        self.diags.error(
                            "R602",
                            span,
                            tr(format!("o campo `{key}` mistura {} e {} nas linhas", known.label(), value.ty().label()), format!("the field `{key}` mixes {} and {} across rows", known.label(), value.ty().label())),
                        );
                        return None;
                    }
                    Some(_) => {}
                    None => {
                        fields.insert(key.clone(), value.ty());
                    }
                }
                row.insert(key.clone(), field);
            }
            rows.push(row);
        }
        Some((rows, fields))
    }

    fn runtime_for(&mut self, directive: &Directive, names: &[&str], list: String, ty: Type) -> Vec<Block> {
        let item = names[0].to_string();
        let index = names.get(1).map(|name| name.to_string());
        let mut frame = Frame::default();
        let binding = if ty == Type::Records {
            Binding::Record(item.clone(), self.record_fields.get(&list).cloned().unwrap_or_default())
        } else {
            Binding::Dynamic(Expr::Item { name: item.clone(), field: None }, Type::Text)
        };
        frame.names.insert(item.clone(), binding);
        if let Some(index) = &index {
            frame.names.insert(index.clone(), Binding::Dynamic(Expr::Item { name: index.clone(), field: None }, Type::Number));
        }
        self.frames.push(frame);
        let children = self.blocks(&directive.children);
        self.frames.pop();
        vec![self.make(BlockKind::Each { list: Expr::State { name: list }, item, index, children }, Resolved::default(), directive.line)]
    }

    pub fn page(&mut self, index: usize, source: &PageSource<'_>) -> Page {
        self.step = 0;
        self.max_step = 0;
        self.uses_elapsed = false;
        self.sounds.clear();
        self.sound_refs.clear();
        self.notes.clear();
        let attrs = source.separator.and_then(|line| separator_attrs(line, self.diags));
        self.current_page = index;
        let mut resolved = self.resolve(attrs.as_ref(), Kind::Page);
        let (definitions, body) = split_definitions(&source.lines);
        self.frames.push(Frame::default());
        let frame = self.frames.len() - 1;
        let mut local_state = BTreeMap::new();
        for line in definitions {
            let trimmed = line.trimmed();
            let keyword = if trimmed.starts_with("@let") { "let" } else { "state" };
            let start = line.indent() + keyword.len() + 1;
            let rest = line.slice(start);
            let value = rest.text_at(rest.indent(), rest.text.trim_end().len().max(rest.indent()));
            if let Some(definition) = assignment(keyword, &value, self.diags) {
                if keyword == "let" {
                    self.define_let(&definition, frame);
                } else if let Some((name, value)) = self.define_state(&definition, frame, false) {
                    local_state.insert(name, value);
                }
            }
        }
        let on_enter = self.enter_actions(&resolved);
        let (timeout, on_timeout) = self.timeout(&resolved);
        let nodes = parse_blocks(body, self.diags);
        self.tracking = true;
        let children = self.blocks(&nodes);
        self.tracking = false;
        self.frames.truncate(frame);
        let title = first_heading(&children).unwrap_or_else(|| tr(format!("Página {}", index + 1), format!("Page {}", index + 1)));
        let id = match resolved.id.take() {
            Some(id) => {
                if !self.page_ids.insert(id.value.clone()) {
                    self.diags.warning("R208", id.span, tr(format!("já existe outra página com o id `#{}`", id.value), format!("another page already uses the id `#{}`", id.value)));
                }
                id.value
            }
            None => format!("page-{}", index + 1),
        };
        let notes = (!self.notes.is_empty()).then(|| self.notes.join("\n\n"));
        let cover = match resolved.extra("cover") {
            Some(pair) => self.switch(pair, "cover").unwrap_or(index == 0),
            None => index == 0,
        };
        let show = |lowerer: &mut Self, key: &str| match resolved.extra(key) {
            Some(pair) => lowerer.switch(pair, key).unwrap_or(!cover),
            None => !cover,
        };
        let (show_header, show_footer, show_overlay) = (show(self, "header"), show(self, "footer"), show(self, "overlay"));
        for key in ["hidden", "next"] {
            if let Some(pair) = resolved.extra(key) {
                self.forbid_in_lesson(&format!("`{key}`"), pair.key.span);
            }
        }
        let hidden = match resolved.extra("hidden") {
            Some(pair) => self.switch(pair, "hidden").unwrap_or(false),
            None => false,
        };
        if hidden {
            self.hidden.push((id.clone(), resolved.extra("hidden").map_or(Span::new(source.start_line, 0, 3), |pair| pair.key.span)));
        }
        let next = resolved.extra("next").cloned().and_then(|pair| self.page_target(&pair, "next"));
        let locked = match resolved.extra("nav").cloned() {
            Some(pair) => self.switch(&pair, "nav").map(|nav| !nav).unwrap_or(self.mode == Mode::Game),
            None => self.mode == Mode::Game,
        };
        if let Some(pair) = resolved.extra("steps").cloned() {
            match pair.value.as_ref().and_then(|value| value.value.parse::<u32>().ok()).filter(|n| (1..=99).contains(n)) {
                Some(steps) => self.max_step = self.max_step.max(steps),
                None => {
                    self.diags.error("R301", pair.value.as_ref().map_or(pair.key.span, |value| value.span), tr("número de etapas inválido", "invalid number of steps")).hint(tr("Use um número de 1 a 99, como steps=3", "Use a number from 1 to 99, such as steps=3"));
                }
            }
        }
        let transition = match resolved.extra("transition").cloned() {
            Some(pair) => match pair.value.as_ref().map(|value| style::transition(&value.value)) {
                Some(Ok(value)) => Some(value),
                Some(Err(invalid)) => {
                    self.report(pair.value.as_ref().map_or(pair.key.span, |value| value.span), invalid);
                    self.transition
                }
                None => self.transition,
            },
            None => self.transition,
        };
        let (width, height) = self.page_dimensions(&resolved);
        let steps = self.max_step;
        let header = if show_header { self.region(Region::Header, index + 1) } else { None };
        let footer = if show_footer { self.region(Region::Footer, index + 1) } else { None };
        let overlay = if show_overlay { self.region(Region::Overlay, index + 1) } else { None };
        self.check_sounds();
        Page {
            id,
            title,
            line: source.start_line,
            end_line: source.end_line,
            width,
            height,
            steps,
            style: resolved.style,
            notes,
            children,
            header,
            footer,
            overlay,
            hidden,
            next,
            state: local_state,
            transition,
            on_enter,
            locked,
            clock: self.uses_elapsed || timeout.is_some(),
            timeout,
            on_timeout,
        }
    }

    fn timeout(&mut self, resolved: &Resolved) -> (Option<f64>, Vec<Action>) {
        let timeout = resolved.extra("timeout").cloned().and_then(|pair| {
            let parsed = pair.value.as_ref().and_then(|value| value.value.trim_end_matches('s').parse::<f64>().ok()).filter(|seconds| *seconds > 0.0 && *seconds <= 3600.0);
            if parsed.is_none() {
                self.diags
                    .error("R301", pair.value.as_ref().map_or(pair.key.span, |value| value.span), tr("tempo inválido para `timeout`", "invalid time for `timeout`"))
                    .hint(tr("Use os segundos, de 1 a 3600, como timeout=30", "Use the seconds, from 1 to 3600, such as timeout=30"));
            }
            parsed
        });
        let pairs: Vec<Pair> = resolved.extra.iter().filter(|pair| pair.key.value == "on-timeout").cloned().collect();
        let mut actions = Vec::new();
        for pair in pairs {
            let Some(value) = &pair.value else {
                self.diags.error("R203", pair.key.span, tr("`on-timeout` precisa de uma ação, como on-timeout=\"go #fim\"", "`on-timeout` needs an action, such as on-timeout=\"go #end\""));
                continue;
            };
            if timeout.is_none() && resolved.extra("timeout").is_none() {
                self.diags
                    .error("R614", pair.key.span, tr("`on-timeout` precisa de `timeout` na mesma página", "`on-timeout` needs `timeout` on the same page"))
                    .hint(tr("Diga quantos segundos esperar: --- {timeout=30 on-timeout=\"go #fim\"}", "Say how many seconds to wait: --- {timeout=30 on-timeout=\"go #end\"}"));
                continue;
            }
            actions.extend(self.actions(value));
        }
        (timeout, actions)
    }

    fn enter_actions(&mut self, resolved: &Resolved) -> Vec<Action> {
        let pairs: Vec<Pair> = resolved.extra.iter().filter(|pair| pair.key.value == "on-enter").cloned().collect();
        let mut actions = Vec::new();
        for pair in pairs {
            let Some(value) = &pair.value else {
                self.diags.error("R203", pair.key.span, tr("`on-enter` precisa de uma ação, como on-enter=\"$visto = true\"", "`on-enter` needs an action, such as on-enter=\"$seen = true\""));
                continue;
            };
            let navigates = expr::parse_actions(&value.value)
                .is_ok_and(|raw| raw.iter().any(|action| matches!(action, RawAction::Go { .. } | RawAction::Back | RawAction::Next | RawAction::Prev)));
            if navigates {
                self.diags
                    .error("R612", value.span, tr("`on-enter` só pode mudar o estado; `go`, `back`, `next` e `prev` não são permitidos", "`on-enter` can only change state; `go`, `back`, `next` and `prev` are not allowed"))
                    .hint(tr("Para escolher a página seguinte, use next=#id no separador", "To choose the following page, use next=#id on the separator"));
                continue;
            }
            actions.extend(self.actions(value));
        }
        actions
    }

    fn page_dimensions(&mut self, resolved: &Resolved) -> (u32, Option<u32>) {
        let mut size = self.page;
        if let Some(pair) = resolved.extra("width").cloned() {
            match pair.value.as_ref().and_then(|value| page_length(&value.value)) {
                Some(width) => size.width = width,
                None => {
                    self.diags.error("R515", pair.value.as_ref().map_or(pair.key.span, |value| value.span), tr("largura de página inválida", "invalid page width")).hint(tr(format!("Use um número de {MIN_PAGE} a {MAX_PAGE}, como width=1080"), format!("Use a number from {MIN_PAGE} to {MAX_PAGE}, such as width=1080")));
                }
            }
        }
        if let Some(pair) = resolved.extra("height").cloned() {
            match pair.value.as_ref().and_then(|value| page_height(&value.value)) {
                Some(height) => size.height = height,
                None => {
                    self.diags.error("R515", pair.value.as_ref().map_or(pair.key.span, |value| value.span), tr("altura de página inválida", "invalid page height")).hint(tr(format!("Use um número de {MIN_PAGE} a {MAX_PAGE} ou auto, como height=1920"), format!("Use a number from {MIN_PAGE} to {MAX_PAGE} or auto, such as height=1920")));
                }
            }
        }
        (size.width, size.height)
    }

    fn page_target(&mut self, pair: &Pair, key: &str) -> Option<String> {
        let Some(value) = &pair.value else {
            self.diags.error("R301", pair.key.span, tr(format!("`{key}` precisa do id de uma página, como {key}=#resumo"), format!("`{key}` needs a page id, such as {key}=#summary")));
            return None;
        };
        let id = value.value.strip_prefix('#').unwrap_or(&value.value).to_string();
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            self.diags.error("R301", value.span, tr(format!("id de página inválido: `{}`", value.value), format!("invalid page id: `{}`", value.value)));
            return None;
        }
        self.targets.push((id.clone(), value.span, true));
        Some(id)
    }

    fn scorm(&mut self, text: &Text) -> Option<Scorm> {
        let keys_hint = || tr("Use @scorm score=$pontos complete=#final, com max= e pass= opcionais", "Use @scorm score=$points complete=#end, with optional max= and pass=");
        let mut scorm = Scorm { score: None, complete: Vec::new(), max: 100.0, pass: None };
        let mut offset = 0;
        let mut tracked = false;
        for word in text.value.split_whitespace() {
            let start = text.value[offset..].find(word).map_or(offset, |found| offset + found);
            offset = start + word.len();
            let span = Span::new(text.span.line, text.span.from + start, text.span.from + offset);
            let Some((key, value)) = word.split_once('=').filter(|(_, value)| !value.is_empty()) else {
                self.diags.error("R618", span, tr(format!("parâmetro inválido em @scorm: `{word}`"), format!("invalid @scorm parameter: `{word}`"))).hint(keys_hint());
                continue;
            };
            let value_span = Span::new(span.line, span.from + key.len() + 1, span.to);
            tracked |= key == "score" || key == "complete";
            match key {
                "score" => scorm.score = self.scorm_score(value.trim_start_matches('$'), value_span),
                "complete" => {
                    for target in value.split(',').filter(|target| !target.is_empty()) {
                        let id = target.trim_start_matches('#');
                        if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                            self.diags.error("R301", value_span, tr(format!("id de página inválido: `{target}`"), format!("invalid page id: `{target}`")));
                            continue;
                        }
                        self.targets.push((id.to_owned(), value_span, true));
                        scorm.complete.push(id.to_owned());
                    }
                }
                "max" | "pass" => match value.parse::<f64>() {
                    Ok(number) if number.is_finite() && number >= 0.0 && (key == "pass" || number > 0.0) => {
                        if key == "max" {
                            scorm.max = number;
                        } else {
                            scorm.pass = Some(number);
                        }
                    }
                    _ => {
                        self.diags.error("R618", value_span, tr(format!("`{key}` em @scorm precisa ser um número positivo"), format!("`{key}` in @scorm must be a positive number")));
                    }
                },
                _ => {
                    self.diags.error("R618", span, tr(format!("parâmetro desconhecido em @scorm: `{key}`"), format!("unknown @scorm parameter: `{key}`"))).hint(keys_hint());
                }
            }
        }
        if scorm.score.is_none() && scorm.complete.is_empty() {
            if !tracked {
                self.diags.error("R618", text.span, tr("@scorm precisa de score=$variavel ou complete=#pagina", "@scorm needs score=$variable or complete=#pagina")).hint(keys_hint());
            }
            return None;
        }
        if scorm.pass.is_some_and(|pass| pass > scorm.max) {
            self.diags.error("R618", text.span, tr("`pass` em @scorm não pode ser maior que `max`", "`pass` in @scorm cannot be greater than `max`"));
        }
        Some(scorm)
    }

    fn scorm_score(&mut self, name: &str, span: Span) -> Option<String> {
        match self.frames[0].names.get(name) {
            Some(Binding::State(_)) if self.externs.iter().any(|known| known == name) => {
                self.report_extern(name, span, ExternUse::Score);
                None
            }
            Some(Binding::State(Type::Number)) => Some(name.to_owned()),
            Some(_) => {
                self.diags
                    .error("R618", span, tr(format!("`score` em @scorm precisa ser uma variável numérica de @state, e `${name}` não é"), format!("`score` in @scorm must be a numeric @state variable, and `${name}` is not")))
                    .hint(tr(format!("Declare a nota com @state {name} = 0"), format!("Declare the score with @state {name} = 0")));
                None
            }
            None => {
                let names: Vec<String> = self.frames[0].names.iter().filter(|(_, binding)| matches!(binding, Binding::State(Type::Number))).map(|(name, _)| name.clone()).collect();
                let pending = self.diags.error("R601", span, tr(format!("variável desconhecida: `${name}`"), format!("unknown variable: `${name}`")));
                match suggest(name, names.iter().map(String::as_str)) {
                    Some(found) => pending.hint(tr(format!("Você quis dizer `${found}`?"), format!("Did you mean `${found}`?"))),
                    None => pending.hint(tr(format!("Declare-a antes: @state {name} = 0"), format!("Declare it first: @state {name} = 0"))),
                };
                None
            }
        }
    }

    pub fn document_state(&self) -> BTreeMap<String, Scalar> {
        self.document_state.clone()
    }

    pub fn finish(&mut self, pages: &[Page]) {
        let ids: Vec<&str> = pages.iter().map(|page| page.id.as_str()).collect();
        let mut seen_anchors = HashSet::new();
        for (anchor, _, span) in &self.anchors {
            if ids.contains(&anchor.as_str()) {
                self.diags
                    .error("R211", *span, tr(format!("o id `#{anchor}` já pertence a uma página"), format!("the id `#{anchor}` already belongs to a page")))
                    .hint(tr("Use um id diferente para o bloco", "Use a different id for the block"));
            } else if !seen_anchors.insert(anchor.as_str()) {
                self.diags
                    .error("R211", *span, tr(format!("já existe outro bloco com o id `#{anchor}`"), format!("another block already uses the id `#{anchor}`")))
                    .hint(tr("Use um id diferente para cada bloco", "Use a different id for each block"));
            }
        }
        let mut reached = HashSet::new();
        for (target, span, strict) in std::mem::take(&mut self.targets) {
            if ids.contains(&target.as_str()) {
                reached.insert(target);
                continue;
            }
            if seen_anchors.contains(target.as_str()) {
                continue;
            }
            let message = (tr(format!("não existe página com o id `#{target}`"), format!("there is no page with the id `#{target}`")),);
            let pending = if strict { self.diags.error("R701", span, message.0) } else { self.diags.warning("R701", span, message.0) };
            match suggest(&target, ids.iter().copied()) {
                Some(found) => pending.hint(tr(format!("Você quis dizer `#{found}`?"), format!("Did you mean `#{found}`?"))),
                None => pending.hint(tr("Dê um id à página no separador: --- {#id}", "Give the page an id on its separator: --- {#id}")),
            };
        }
        for (id, span) in std::mem::take(&mut self.hidden) {
            if !reached.contains(&id) {
                self.diags
                    .warning("R702", span, tr(format!("a página oculta `#{id}` não é alcançada por nenhum link ou `go`"), format!("hidden page `#{id}` is not reached by any link or `go`")))
                    .hint(tr(format!("Crie um link para ela, como [Ver detalhes](#{id}) ou {{click=\"go #{id}\"}}"), format!("Link to it, such as [See details](#{id}) or {{click=\"go #{id}\"}}")));
            }
        }
        for (state, value, span) in std::mem::take(&mut self.comparisons) {
            let Some(Some(known)) = self.assigned.get(&state) else {
                continue;
            };
            if known.contains(&value) {
                continue;
            }
            let pending = self.diags.warning("R608", span, tr(format!("`${state}` nunca recebe o valor `{value}`"), format!("`${state}` never gets the value `{value}`")));
            match suggest(&value, known.iter().map(String::as_str)) {
                Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                None if known.is_empty() => pending.hint(tr(format!("Nenhuma ação adiciona valores a `${state}`; use add ${state} {value}"), format!("No action adds values to `${state}`; use add ${state} {value}"))),
                None => pending.hint(tr(format!("Valores usados: {}", known.iter().cloned().collect::<Vec<_>>().join(", ")), format!("Values used: {}", known.iter().cloned().collect::<Vec<_>>().join(", ")))),
            };
        }
    }

    pub fn check_regions(&mut self) {
        for region in [Region::Header, Region::Footer, Region::Overlay] {
            let pending = match region {
                Region::Header => &self.header,
                Region::Footer => &self.footer,
                Region::Overlay => &self.overlay,
            };
            if pending.as_ref().is_some_and(|source| !source.reported) {
                self.region(region, 1);
            }
        }
    }

    fn switch(&mut self, pair: &Pair, key: &str) -> Option<bool> {
        match pair.value.as_ref().map(|value| value.value.as_str()) {
            None | Some("on" | "true" | "yes" | "sim") => Some(true),
            Some("off" | "false" | "no" | "não") => Some(false),
            Some(other) => {
                let span = pair.value.as_ref().map_or(pair.key.span, |value| value.span);
                self.diags
                    .error("R301", span, tr(format!("valor inválido para `{key}`: `{other}`"), format!("invalid value for `{key}`: `{other}`")))
                    .hint(tr(format!("Use {key}=on ou {key}=off"), format!("Use {key}=on or {key}=off")));
                None
            }
        }
    }

    fn region(&mut self, region: Region, page: usize) -> Option<Block> {
        let slot = match region {
            Region::Header => &mut self.header,
            Region::Footer => &mut self.footer,
            Region::Overlay => &mut self.overlay,
        };
        let source = slot.as_mut()?;
        let report = !source.reported;
        source.reported = true;
        let source = source.clone();
        let nodes = source.def.nodes.as_ref()?;
        let name = region.name();
        let hidden = (!report).then(|| std::mem::take(self.diags));

        let mut frame = Frame { barrier: Some(Barrier::Region), ..Frame::default() };
        for (key, value) in [
            ("page", page.to_string()),
            ("pages", self.total_pages.max(page).to_string()),
            ("title", self.title.clone()),
            ("author", self.author.clone()),
            ("date", format!("{DATE_MARK}date{DATE_MARK}")),
        ] {
            frame.names.insert(key.to_string(), Binding::Value(Value::Text(value)));
        }

        let (step, max_step, notes) = (self.step, self.max_step, self.notes.len());
        let resolved = self.resolve(source.def.attrs.as_ref(), Kind::Layout(LayoutKind::Stack));
        self.frames.push(frame);
        let mut children = self.blocks(nodes);
        let frame = self.frames.pop().expect("pushed above");
        let mut unknown: BTreeSet<String> = frame.missing.into_iter().chain(frame.slot_names).collect();
        if frame.used_content {
            unknown.insert("content".to_string());
        }
        for field in unknown {
            let pending = self.diags.error("R522", source.def.keyword, tr(format!("`@{name}` usa `${field}`, que não é um campo disponível"), format!("`@{name}` uses `${field}`, which is not an available field")));
            match field_alias(&field).or_else(|| suggest(&field, REGION_FIELDS.iter().copied())) {
                Some(found) => pending.hint(tr(format!("Você quis dizer `${found}`?"), format!("Did you mean `${found}`?"))),
                None => pending.hint(tr(
                    format!("Campos disponíveis: {}", REGION_FIELDS.iter().map(|field| format!("${field}")).collect::<Vec<_>>().join(", ")),
                    format!("Available fields: {}", REGION_FIELDS.iter().map(|field| format!("${field}")).collect::<Vec<_>>().join(", ")),
                )),
            };
        }

        let cleared = children.iter_mut().fold(false, |found, block| clear_steps(block) | found);
        let had_steps = cleared || self.max_step > max_step;
        if had_steps || resolved.step.is_some() {
            self.diags.warning("R523", source.def.keyword, tr(format!("`@{name}` aparece sempre inteiro; `step` e `::: steps` são ignorados nele"), format!("`@{name}` is always shown in full; `step` and `::: steps` are ignored in it")));
        }
        if self.notes.len() > notes {
            self.diags.warning("R524", source.def.keyword, tr(format!("notas (`::: notes`) dentro de `@{name}` são ignoradas"), format!("notes (`::: notes`) inside `@{name}` are ignored"))).hint(tr("Coloque `::: notes` dentro da página", "Put `::: notes` inside the page"));
            self.notes.truncate(notes);
        }
        self.step = step;
        self.max_step = max_step;
        if let Some(reported) = hidden {
            *self.diags = reported;
        }

        let mut style = resolved.style;
        if region != Region::Overlay {
            if style.size.is_none() {
                style.size = style::size("sm").ok();
            }
            if style.color.is_none() {
                style.color = style::color("muted", &self.theme).ok();
            }
        }
        let line = if source.from_prelude { 0 } else { source.def.keyword.line };
        let mut block = Block { kind: BlockKind::Layout { layout: LayoutKind::Stack, cols: None, widths: None, wrap: false, children }, id: None, style, step: None, behavior: Behavior::default(), line, end_line: None, source: None };
        if source.from_prelude {
            relocate(&mut block);
        }
        Some(block)
    }

    pub fn theme_ir(&self) -> ThemeIr {
        let theme = &self.theme;
        let family = |token: &str| theme.fonts.get(token).map(|spec| spec.family.clone());
        let color = |token: &str| theme.colors.get(token).cloned();
        let mut imports: BTreeMap<String, BTreeSet<u16>> = BTreeMap::new();
        for spec in theme.fonts.values() {
            imports.entry(spec.family.clone()).or_default().extend(spec.weights.iter().copied());
        }
        let font_families = imports.into_iter().map(|(family, weights)| FontFamily { family, weights: weights.into_iter().collect() }).collect();
        ThemeIr {
            name: theme.name.clone(),
            colors: theme.colors.clone(),
            fonts: theme.fonts.iter().map(|(token, spec)| (token.clone(), spec.family.clone())).collect(),
            font_families,
            text: Style {
                color: color("text"),
                font: family("body"),
                size: Some(theme.text_size),
                leading: Some(theme.leading),
                weight: Some(400),
                ..Style::default()
            },
            headings: theme
                .headings
                .iter()
                .map(|heading| Style {
                    size: Some(heading.size),
                    weight: Some(heading.weight),
                    font: family(heading.font),
                    leading: Some(heading.leading),
                    ..Style::default()
                })
                .collect(),
            page: Style {
                bg: color("bg").map(|color| Paint::Solid { color }),
                color: color("text"),
                pad: Some(theme.page_pad),
                gap: Some(theme.gap),
                align: Some(HAlign::Left),
                valign: Some(VAlign::Center),
                ..Style::default()
            },
            palette: theme.palette.iter().filter_map(|token| color(token)).collect(),
            motion: standard_motion(),
            elements: standard_elements(theme),
            layout: standard_layout(),
        }
    }

    fn report(&mut self, span: Span, invalid: Invalid) {
        let pending = self.diags.error("R301", span, invalid.message);
        if let Some(hint) = invalid.hint {
            pending.hint(hint);
        }
    }

    fn visible_frames(&self) -> Vec<&Frame> {
        let mut visible = Vec::new();
        for frame in self.frames[1..].iter().rev() {
            visible.push(frame);
            if frame.barrier.is_some() {
                break;
            }
        }
        visible.push(&self.frames[0]);
        visible
    }

    fn lookup(&self, name: &str) -> Option<Binding> {
        self.visible_frames().into_iter().find_map(|frame| frame.names.get(name).cloned())
    }

    fn barrier_frame(&mut self) -> Option<&mut Frame> {
        self.frames[1..].iter_mut().rev().find(|frame| frame.barrier.is_some())
    }

    fn report_expr(&mut self, origin: &Text, error: ExprError) {
        let span = Span::new(origin.span.line, origin.span.from + error.from, origin.span.from + error.to.max(error.from + 1));
        let pending = if error.code == "R608" { self.diags.warning(error.code, span, error.message) } else { self.diags.error(error.code, span, error.message) };
        if let Some(hint) = error.hint {
            pending.hint(hint);
        }
    }

    fn fold_ast(&mut self, origin: &Text, ast: &Ast) -> Option<Folded> {
        let previous = std::mem::replace(&mut self.expr_origin, origin.span);
        let result = eval::fold(self, ast);
        self.expr_origin = previous;
        match result {
            Ok(folded) => Some(folded),
            Err(error) => {
                self.report_expr(origin, error);
                None
            }
        }
    }

    fn expression(&mut self, text: &Text) -> Option<Folded> {
        match expr::parse(&text.value) {
            Ok(ast) => self.fold_ast(text, &ast),
            Err(error) => {
                self.report_expr(text, error);
                None
            }
        }
    }

    fn condition(&mut self, text: &Text) -> Option<Folded> {
        let folded = self.expression(text)?;
        if folded.ty() != Type::Bool {
            self.diags
                .error("R602", text.span, tr(format!("a condição precisa ser verdadeiro/falso, mas o valor é {}", folded.ty().label()), format!("the condition must be true/false, but the value is {}", folded.ty().label())))
                .hint(tr("Compare o valor, como $total > 0 ou $plano == pro", "Compare the value, such as $total > 0 or $plan == pro"));
            return None;
        }
        Some(folded)
    }

    fn marker(&mut self, expr: Expr) -> String {
        self.dynamic.push(expr);
        format!("{DYNAMIC_MARK}{}{DYNAMIC_MARK}", self.dynamic.len() - 1)
    }

    fn dynamic_here(&mut self, span: Span) {
        self.diags
            .error("R604", span, tr("este valor depende do estado do documento e não pode ser usado aqui", "this value depends on the document state and cannot be used here"))
            .hint(tr("Para mudar o visual conforme o estado, use uma variante: {.destaque=$ativo}", "To change the look based on state, use a variant: {.highlight=$active}"));
    }

    fn render_folded(&mut self, folded: Folded, span: Span, allow_dynamic: bool) -> Option<String> {
        match folded {
            Folded::Static(value) => match value.display() {
                Some(text) => Some(text),
                None => {
                    self.diags
                        .error("R602", span, tr("um registro não pode ser exibido como texto; use um campo, como $pessoa.nome", "a record cannot be shown as text; use a field, such as $person.name"));
                    None
                }
            },
            Folded::Dynamic(_, Type::Records) => {
                self.diags
                    .error("R602", span, tr("uma lista de registros não vira texto", "a list of records cannot be shown as text"))
                    .hint(tr("Percorra a lista com ::: for item in $lista e mostre os campos, como $item.nome", "Loop over it with ::: for item in $list and show the fields, such as $item.name"));
                None
            }
            Folded::Dynamic(expr, _) => {
                if allow_dynamic {
                    Some(self.marker(expr))
                } else {
                    self.dynamic_here(span);
                    None
                }
            }
        }
    }

    fn interpolate(&mut self, text: &Text, allow_dynamic: bool) -> (String, bool) {
        let source = text.value.as_str();
        if !source.contains('$') {
            return (source.to_string(), false);
        }
        let bytes = source.as_bytes();
        let mut out = String::with_capacity(source.len());
        let mut changed = false;
        let mut i = 0;
        while i < bytes.len() {
            let byte = bytes[i];
            if byte == b'\\' && i + 1 < bytes.len() {
                let next = source[i + 1..].chars().next().expect("in bounds");
                out.push('\\');
                out.push(next);
                i += 1 + next.len_utf8();
                continue;
            }
            if byte == b'{' && i > 0 && matches!(bytes[i - 1], b']' | b')')
                && let Some(close) = matching_brace(bytes, i) {
                    out.push_str(&source[i..=close]);
                    i = close + 1;
                    continue;
                }
            if byte != b'$' {
                let c = source[i..].chars().next().expect("in bounds");
                out.push(c);
                i += c.len_utf8();
                continue;
            }
            let at = |from: usize, to: usize| Span::new(text.span.line, text.span.from + from, text.span.from + to);
            if bytes.get(i + 1) == Some(&b'(') {
                let Some(close) = closing_paren(source, i + 1) else {
                    self.diags.error("R603", at(i, source.len()), tr("`$(` sem o `)` de fechamento", "`$(` without the closing `)`"));
                    out.push_str(&source[i..]);
                    break;
                };
                let inner = Text::new(&source[i + 2..close], text.span.line, text.span.from + i + 2);
                match self.expression(&inner).and_then(|folded| self.render_folded(folded, at(i, close + 1), allow_dynamic)) {
                    Some(value) => {
                        out.push_str(&value);
                        changed = true;
                    }
                    None => {
                        self.interpolation_failed = true;
                        out.push_str(&source[i..=close]);
                    }
                }
                i = close + 1;
                continue;
            }
            let Some((path, end)) = path_at(source, i + 1) else {
                out.push('$');
                i += 1;
                continue;
            };
            let name = path[0].clone();
            let raw = &source[i..end];
            if name == "content" && path.len() == 1 {
                out.push_str(raw);
                i = end;
                continue;
            }
            match self.lookup(&name) {
                None => {
                    if let Some(frame) = self.barrier_frame() {
                        frame.missing.insert(name);
                    }
                    out.push_str(raw);
                }
                Some(_) => {
                    let ast = Ast { kind: AstKind::Var(path), from: 0, to: raw.len() };
                    let origin = Text::new(raw, text.span.line, text.span.from + i);
                    match self.fold_ast(&origin, &ast).and_then(|folded| self.render_folded(folded, origin.span, allow_dynamic)) {
                        Some(value) => {
                            out.push_str(&value);
                            changed = true;
                        }
                        None => {
                            self.interpolation_failed = true;
                            out.push_str(raw);
                        }
                    }
                }
            }
            i = end;
        }
        (out, changed)
    }

    fn interpolated(&mut self, text: &Text, allow_dynamic: bool) -> Text {
        Text { value: self.interpolate(text, allow_dynamic).0, span: text.span }
    }

    fn actions(&mut self, text: &Text) -> Vec<Action> {
        let previous = std::mem::replace(&mut self.in_action, true);
        let actions = self.lower_actions(text);
        self.in_action = previous;
        actions
    }

    fn lower_actions(&mut self, text: &Text) -> Vec<Action> {
        let raw = match expr::parse_actions(&text.value) {
            Ok(raw) => raw,
            Err(error) => {
                self.report_expr(text, error);
                return Vec::new();
            }
        };
        let mut actions = Vec::new();
        for action in raw {
            let adding = matches!(action, RawAction::Add { .. });
            let playing = matches!(action, RawAction::Play { .. });
            match action {
                RawAction::Back => actions.push(Action::Back),
                RawAction::Next => actions.push(Action::Next),
                RawAction::Prev => actions.push(Action::Prev),
                RawAction::Go { target, from, to } => {
                    let span = Span::new(text.span.line, text.span.from + from, text.span.from + to);
                    let target = if target.contains('$') { self.interpolate(&Text { value: target, span }, false).0 } else { target };
                    if target.contains('$') || !target.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                        self.diags.error("R611", span, tr(format!("destino inválido: `#{target}`"), format!("invalid destination: `#{target}`")));
                        continue;
                    }
                    self.targets.push((target.clone(), span, true));
                    actions.push(Action::Go { target });
                }
                RawAction::Toggle { name, from, to } => {
                    let span = Span::new(text.span.line, text.span.from + from, text.span.from + to);
                    match self.state_binding(&name) {
                        Some((real, Type::Bool)) => actions.push(Action::Toggle { name: real }),
                        _ => {
                            self.state_target(&name, Some(Type::Bool), "toggle", span);
                        }
                    }
                }
                RawAction::Play { name, from, to } | RawAction::Stop { name, from, to } => {
                    let span = Span::new(text.span.line, text.span.from + from, text.span.from + to);
                    self.sound_refs.push((name.clone(), span));
                    actions.push(if playing { Action::Play { name } } else { Action::Stop { name } });
                }
                RawAction::Clear { name, from, to } => {
                    let span = Span::new(text.span.line, text.span.from + from, text.span.from + to);
                    match self.state_binding(&name) {
                        Some((real, Type::List)) => actions.push(Action::Clear { name: real }),
                        _ => {
                            self.state_target(&name, Some(Type::List), "clear", span);
                        }
                    }
                }
                RawAction::Add { name, from, to, value } | RawAction::Remove { name, from, to, value } => {
                    let span = Span::new(text.span.line, text.span.from + from, text.span.from + to);
                    let Some((real, Type::List)) = self.state_binding(&name) else {
                        self.state_target(&name, Some(Type::List), if adding { "add" } else { "remove" }, span);
                        continue;
                    };
                    let Some(folded) = self.fold_ast(text, &value) else {
                        continue;
                    };
                    if folded.ty() != Type::Text {
                        self.diags.error(
                            "R602",
                            Span::new(text.span.line, text.span.from + value.from, text.span.from + value.to),
                            tr(format!("listas do estado guardam textos, mas o valor é {}", folded.ty().label()), format!("state lists hold text, but the value is {}", folded.ty().label())),
                        );
                        continue;
                    }
                    if adding {
                        match &folded {
                            Folded::Static(Value::Text(value)) => {
                                if let Some(Some(known)) = self.assigned.get_mut(&real) {
                                    known.insert(value.clone());
                                }
                            }
                            _ => {
                                self.assigned.insert(real.clone(), None);
                            }
                        }
                    }
                    if let Some(expr) = folded.into_expr() {
                        actions.push(if adding { Action::Add { name: real, value: expr } } else { Action::Remove { name: real, value: expr } });
                    }
                }
                RawAction::Set { name, name_from, name_to, value } => {
                    let span = Span::new(text.span.line, text.span.from + name_from, text.span.from + name_to);
                    let Some((name, ty)) = self.state_binding(&name).or_else(|| {
                        self.state_target(&name, None, "", span);
                        None
                    }) else {
                        continue;
                    };
                    if ty == Type::List {
                        self.diags
                            .error("R602", span, tr(format!("`${name}` é uma lista e não recebe `=`"), format!("`${name}` is a list and cannot be assigned with `=`")))
                            .hint(tr(format!("Use add ${name} valor, remove ${name} valor ou clear ${name}"), format!("Use add ${name} value, remove ${name} value or clear ${name}")));
                        continue;
                    }
                    let Some(folded) = self.fold_ast(text, &value) else {
                        continue;
                    };
                    if folded.ty() != ty {
                        self.diags.error(
                            "R602",
                            Span::new(text.span.line, text.span.from + value.from, text.span.from + value.to),
                            tr(format!("`${name}` guarda {}, mas recebeu {}", ty.label(), folded.ty().label()), format!("`${name}` holds {}, but got {}", ty.label(), folded.ty().label())),
                        );
                        continue;
                    }
                    match &folded {
                        Folded::Static(Value::Text(value)) => {
                            if let Some(Some(known)) = self.assigned.get_mut(&name) {
                                known.insert(value.clone());
                            }
                        }
                        Folded::Dynamic(..) => {
                            self.assigned.insert(name.clone(), None);
                        }
                        Folded::Static(_) => {}
                    }
                    if let Some(expr) = folded.into_expr() {
                        actions.push(Action::Set { name, value: expr });
                    }
                }
            }
        }
        actions
    }

    fn state_binding(&self, name: &str) -> Option<(String, Type)> {
        let found = match self.lookup(name) {
            Some(Binding::State(ty)) => Some((name.to_string(), ty)),
            Some(Binding::Dynamic(Expr::State { name: real }, ty)) => Some((real, ty)),
            _ => None,
        };
        found.filter(|(real, _)| !self.externs.contains(real))
    }

    fn extern_behind(&self, name: &str) -> Option<String> {
        let real = match self.lookup(name) {
            Some(Binding::State(_)) => name.to_string(),
            Some(Binding::Dynamic(Expr::State { name: real }, _)) => real,
            _ => return None,
        };
        self.externs.contains(&real).then_some(real)
    }

    fn report_extern(&mut self, name: &str, span: Span, usage: ExternUse) {
        let (pt, en) = match usage {
            ExternUse::Change => (format!("`${name}` vem de fora do documento (@extern) e não muda por ações"), format!("`${name}` comes from outside the document (@extern) and is not changed by actions")),
            ExternUse::Persist => (format!("`@persist` não guarda `${name}`, que vem de fora do documento (@extern)"), format!("`@persist` does not keep `${name}`, which comes from outside the document (@extern)")),
            ExternUse::Score => (format!("`score` em @scorm não pode ser `${name}`, que vem de fora do documento (@extern)"), format!("`score` in @scorm cannot be `${name}`, which comes from outside the document (@extern)")),
        };
        self.diags
            .error("R619", span, tr(pt, en))
            .hint(tr("Quem exibe o documento define esse valor; para um valor que o próprio documento muda, use @state", "The host that shows the document sets this value; for a value the document changes itself, use @state"));
    }

    fn state_target(&mut self, name: &str, ty: Option<Type>, verb: &str, span: Span) -> bool {
        if let Some(real) = self.extern_behind(name) {
            self.report_extern(&real, span, ExternUse::Change);
            return false;
        }
        match self.lookup(name) {
            Some(Binding::State(actual)) => match ty {
                Some(expected) if expected != actual => {
                    let (needs_pt, needs_en) = if expected == Type::List { ("listas", "lists") } else { ("verdadeiro/falso", "true/false") };
                    self.diags.error(
                        "R602",
                        span,
                        tr(format!("`{verb}` só funciona com {needs_pt}, mas `${name}` guarda {}", actual.label()), format!("`{verb}` only works with {needs_en}, but `${name}` holds {}", actual.label())),
                    );
                    false
                }
                _ => true,
            },
            Some(_) => {
                self.diags
                    .error("R605", span, tr(format!("`${name}` não pode ser alterado"), format!("`${name}` cannot be changed")))
                    .hint(tr("Só variáveis declaradas com @state mudam durante o documento", "Only variables declared with @state change while the document runs"));
                false
            }
            None => {
                let names = self.names();
                let pending = self.diags.error("R601", span, tr(format!("variável desconhecida: `${name}`"), format!("unknown variable: `${name}`")));
                match suggest(name, names.iter().map(String::as_str)) {
                    Some(found) => pending.hint(tr(format!("Você quis dizer `${found}`?"), format!("Did you mean `${found}`?"))),
                    None => pending.hint(tr(format!("Declare-a no início do documento: @state {name} = ..."), format!("Declare it at the start of the document: @state {name} = ..."))),
                };
                false
            }
        }
    }

    fn resolve(&mut self, attrs: Option<&Attrs>, kind: Kind) -> Resolved {
        let mut resolved = Resolved::default();
        let Some(attrs) = attrs else {
            return resolved;
        };
        let mut classes: Vec<Text> = attrs.classes.iter().map(|class| self.interpolated(class, false)).collect();
        let mut variants: Vec<(Text, Expr)> = Vec::new();
        for (class, condition) in &attrs.conditional {
            let class = self.interpolated(class, false);
            if condition.value.trim() == "hover" {
                variants.push((class, Expr::Hover));
                continue;
            }
            match self.condition(condition) {
                Some(Folded::Static(Value::Bool(true))) => classes.push(class),
                Some(Folded::Dynamic(expr, _)) => variants.push((class, expr)),
                _ => {
                    let mut ignored = Vec::new();
                    self.expand_class(&class, &mut ignored, 0);
                }
            }
        }
        let mut inherited = Vec::new();
        for class in &classes {
            self.expand_class(class, &mut inherited, 0);
        }
        for pair in &inherited {
            self.apply(pair, kind, &mut resolved, true);
        }
        for pair in &attrs.pairs {
            self.apply(pair, kind, &mut resolved, false);
        }
        for (class, when) in variants {
            let mut pairs = Vec::new();
            self.expand_class(&class, &mut pairs, 0);
            let mut variant = Resolved::default();
            for pair in &pairs {
                self.apply(pair, kind, &mut variant, true);
            }
            resolved.behavior.variants.push(Variant { when, style: variant.style });
        }
        self.check_key(&resolved);
        if let Some(id) = &attrs.id {
            if !matches!(kind, Kind::Span | Kind::Any) {
                resolved.id = Some(id.clone());
            } else {
                self.diags.warning("R205", id.span, tr("`#id` só tem efeito no separador de página e no fim de blocos", "`#id` only works on the page separator and at the end of blocks"));
            }
        }
        resolved
    }

    fn check_key(&mut self, resolved: &Resolved) {
        if let Some(span) = resolved.key_span
            && resolved.behavior.click.is_empty()
        {
            self.diags
                .warning("R210", span, tr("`key` não faz nada sem `click`", "`key` does nothing without `click`"))
                .hint(tr("A tecla aciona as ações de click: [Norte]{click=\"$y = $y - 1\" key=up}", "The key runs the click actions: [North]{click=\"$y = $y - 1\" key=up}"));
        }
    }

    fn expand_class(&mut self, class: &Text, out: &mut Vec<Pair>, depth: usize) {
        if depth > 8 {
            self.diags.error("R206", class.span, tr(format!("o estilo `.{}` referencia a si mesmo", class.value), format!("style `.{}` references itself", class.value)));
            return;
        }
        if let Some(attrs) = self.styles.get(&class.value).cloned() {
            for nested in &attrs.classes {
                self.expand_class(nested, out, depth + 1);
            }
            out.extend(attrs.pairs.iter().cloned());
        } else if self.theme.colors.contains_key(&class.value) {
            out.push(Pair { key: Text::new("color", class.span.line, class.span.from), value: Some(class.clone()) });
        } else {
            let candidates: Vec<String> = self.styles.keys().chain(self.theme.colors.keys()).cloned().collect();
            let pending = self.diags.error("R207", class.span, tr(format!("estilo desconhecido: `.{}`", class.value), format!("unknown style: `.{}`", class.value)));
            match suggest(&class.value, candidates.iter().map(String::as_str)) {
                Some(found) => pending.hint(tr(format!("Você quis dizer `.{found}`?"), format!("Did you mean `.{found}`?"))),
                None => pending.hint(tr("Declare o estilo no início do documento com @style nome {atributos}", "Declare the style at the start of the document with @style name {attributes}")),
            };
        }
    }

    fn apply(&mut self, pair: &Pair, kind: Kind, resolved: &mut Resolved, silent: bool) {
        let key = pair.key.value.as_str();
        let hidden = silent.then(|| std::mem::take(self.diags));
        self.interpolation_failed = false;
        let interpolated = match &pair.value {
            Some(value) if !EXPRESSION_KEYS.contains(&key) => Some(self.interpolated(value, false)),
            other => other.clone(),
        };
        if std::mem::take(&mut self.interpolation_failed) {
            if let Some(reported) = hidden {
                *self.diags = reported;
            }
            return;
        }
        let pair = &Pair { key: pair.key.clone(), value: interpolated };
        let value = pair.value.as_ref().map(|value| value.value.as_str());
        let span = pair.value.as_ref().map_or(pair.key.span, |value| value.span);
        if key == "if" && kind != Kind::Page {
            match pair.value.clone() {
                Some(value) => match self.condition(&value) {
                    Some(Folded::Static(Value::Bool(false))) => resolved.dropped = true,
                    Some(Folded::Dynamic(expr, _)) => resolved.behavior.when = Some(and(resolved.behavior.when.take(), expr)),
                    _ => {}
                },
                None => {
                    self.diags.error("R203", pair.key.span, tr("`if` precisa de uma condição, como if=$detalhes", "`if` needs a condition, such as if=$details"));
                }
            }
        } else if key == "key" && kind != Kind::Page {
            match pair.value.as_ref().and_then(|value| shortcut(&value.value)) {
                Some(shortcut) => {
                    resolved.behavior.key = Some(shortcut);
                    resolved.key_span = Some(pair.key.span);
                }
                None => {
                    let shown = pair.value.as_ref().map(|value| value.value.clone()).unwrap_or_default();
                    self.diags
                        .error("R301", span, tr(format!("tecla inválida: `{shown}`"), format!("invalid key: `{shown}`")))
                        .hint(tr("Use uma letra, um número, up, down, left, right, Enter ou Space, como key=up", "Use a letter, a digit, up, down, left, right, Enter or Space, such as key=up"));
                }
            }
        } else if key == "click" && kind != Kind::Page {
            match pair.value.clone() {
                Some(value) => {
                    let actions = self.actions(&value);
                    resolved.behavior.click.extend(actions);
                }
                None => {
                    self.diags.error("R203", pair.key.span, tr("`click` precisa de uma ação, como click=\"go #resumo\"", "`click` needs an action, such as click=\"go #summary\""));
                }
            }
        } else {
            let theme = &self.theme;
            let style = &mut resolved.style;
            let outcome: Parsed<()> = match key {
                "color" => needs(value, key).and_then(|v| style::color(v, theme)).map(|v| style.color = Some(v)),
                "bg" => needs(value, key).and_then(|v| style::background(v, theme)).map(|v| style.bg = Some(v)),
                "image" => needs(value, key).and_then(style::image_source).map(|v| style.bg_image = Some(v)),
                "font" => needs(value, key).and_then(|v| style::font(v, theme)).map(|v| style.font = Some(v)),
                "size" => needs(value, key).and_then(style::size).map(|v| style.size = Some(v)),
                "weight" => needs(value, key).and_then(style::weight).map(|v| style.weight = Some(v)),
                "italic" => style::flag(value).map(|v| style.italic = Some(v)),
                "uppercase" => style::flag(value).map(|v| style.uppercase = Some(v)),
                "underline" => style::flag(value).map(|v| style.underline = Some(v)),
                "align" => needs(value, key).and_then(style::halign).map(|v| style.align = Some(v)),
                "valign" => needs(value, key).and_then(style::valign).map(|v| style.valign = Some(v)),
                "pad" => needs(value, key).and_then(style::pad).map(|v| style.pad = Some(v)),
                "gap" => needs(value, key).and_then(style::space).map(|v| style.gap = Some(v)),
                "radius" => needs(value, key).and_then(style::radius).map(|v| style.radius = Some(v)),
                "border" => style::border(value, theme).map(|v| style.border = Some(v)),
                "shadow" => style::shadow(value).map(|v| style.shadow = Some(v)),
                "w" => needs(value, key).and_then(|v| style::length(v, true)).map(|v| style.w = Some(v)),
                "h" => needs(value, key).and_then(|v| style::length(v, true)).map(|v| style.h = Some(v)),
                "x" => needs(value, key).and_then(|v| style::length(v, false)).map(|v| style.x = Some(v)),
                "y" => needs(value, key).and_then(|v| style::length(v, false)).map(|v| style.y = Some(v)),
                "opacity" => needs(value, key).and_then(style::opacity).map(|v| style.opacity = Some(v)),
                "leading" => needs(value, key).and_then(style::leading).map(|v| style.leading = Some(v)),
                "tracking" => needs(value, key).and_then(style::tracking).map(|v| style.tracking = Some(v)),
                "shrink" => style::shrink(value).map(|v| style.shrink = v),
                "enter" => needs(value, key).and_then(style::enter).map(|v| style.enter = Some(v)),
                "step" => needs(value, key)
                    .and_then(|v| {
                        v.parse::<u32>().ok().filter(|n| (1..=99).contains(n)).ok_or_else(|| Invalid {
                            message: tr(format!("etapa inválida: `{v}`"), format!("invalid step: `{v}`")),
                            hint: Some(tr("Use um número inteiro a partir de 1", "Use a whole number starting at 1")),
                        })
                    })
                    .map(|v| resolved.step = Some(v)),
                known if kind.extra_keys().contains(&known) => {
                    resolved.extra.push(pair.clone());
                    Ok(())
                }
                unknown => {
                    if !silent {
                        let candidates: Vec<&str> = STYLE_KEYS.iter().filter(|key| kind != Kind::Page || !matches!(**key, "if" | "click" | "key")).chain(kind.extra_keys()).copied().collect();
                        let pending = self.diags.error("R209", pair.key.span, tr(format!("atributo desconhecido: `{unknown}`"), format!("unknown attribute: `{unknown}`")));
                        if unknown == "on-enter" {
                            pending.hint(tr("`on-enter` só vale no separador da página: --- {on-enter=\"$visto = true\"}", "`on-enter` only works on the page separator: --- {on-enter=\"$seen = true\"}"));
                        } else {
                            match suggest(unknown, candidates.iter().copied()) {
                                Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                                None => pending.hint(tr(format!("Atributos aceitos aqui: {}", candidates.join(", ")), format!("Attributes accepted here: {}", candidates.join(", ")))),
                            };
                        }
                    }
                    Ok(())
                }
            };
            if let Err(invalid) = outcome {
                self.report(span, invalid);
            }
        }
        if let Some(reported) = hidden {
            *self.diags = reported;
        }
    }

    fn note_step(&mut self, step: Option<u32>) {
        if let Some(step) = step {
            self.step = self.step.max(step);
            self.max_step = self.max_step.max(step);
        }
    }

    fn make(&mut self, kind: BlockKind, resolved: Resolved, line: usize) -> Block {
        self.note_step(resolved.step);
        let mut behavior = resolved.behavior;
        if resolved.dropped {
            behavior.when = Some(dropped());
        }
        let id = if resolved.dropped { None } else { self.register_anchor(resolved.id.as_ref()) };
        Block { kind, id, style: resolved.style, step: resolved.step, behavior, line, end_line: None, source: None }
    }

    fn register_anchor(&mut self, id: Option<&Text>) -> Option<String> {
        let id = id?;
        self.anchors.push((id.value.clone(), self.current_page, id.span));
        Some(id.value.clone())
    }

    pub fn anchor_pages(&self) -> BTreeMap<String, usize> {
        let mut pages = BTreeMap::new();
        for (id, page, _) in &self.anchors {
            pages.entry(id.clone()).or_insert(*page);
        }
        pages
    }

    fn source_of(&self, lines: &[Text]) -> Option<SourceRange> {
        if !self.tracking {
            return None;
        }
        let line = lines.first()?.span.line;
        let consecutive = lines.iter().enumerate().all(|(index, text)| text.span.line == line + index);
        consecutive.then(|| SourceRange {
            line,
            spans: lines
                .iter()
                .map(|text| {
                    let from = text.span.from + (text.value.len() - text.value.trim_start().len());
                    [from, text.span.to.max(from)]
                })
                .collect(),
        })
    }

    fn inline(&mut self, text: &Text) -> Vec<Inline> {
        let mut local = Diagnostics::default();
        let raw = parse_inline(&text.value, text.span.line, text.span.from, &mut local);
        self.diags.absorb(local);
        self.convert(&raw)
    }

    fn inline_lines(&mut self, lines: &[Text]) -> (Vec<Inline>, bool) {
        let mut out = Vec::new();
        let mut changed = false;
        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                out.push(Inline::Break);
            }
            let (value, line_changed) = self.interpolate(line, true);
            changed |= line_changed;
            out.extend(self.inline(&Text { value, span: line.span }));
        }
        (out, changed)
    }

    fn convert(&mut self, nodes: &[RawInline]) -> Vec<Inline> {
        let mut out = Vec::new();
        for node in nodes {
            match node {
                RawInline::Text(value) => push_text(&mut out, value, &self.dynamic),
                RawInline::Code(value) => out.push(Inline::Code { value: value.clone() }),
                RawInline::Strong(children) => out.push(Inline::Strong { children: self.convert(children) }),
                RawInline::Emphasis(children) => out.push(Inline::Emphasis { children: self.convert(children) }),
                RawInline::Strike(children) => out.push(Inline::Strike { children: self.convert(children) }),
                RawInline::Link { href, span, children } => {
                    let children = self.convert(children);
                    if href.contains(DYNAMIC_MARK) {
                        self.dynamic_here(*span);
                        out.extend(children);
                        continue;
                    }
                    match style::link(href) {
                        Ok(href) => {
                            if let Some(target) = href.strip_prefix('#') {
                                self.targets.push((target.to_string(), *span, false));
                            }
                            out.push(Inline::Link { href, children });
                        }
                        Err(invalid) => {
                            self.report(*span, invalid);
                            out.extend(children);
                        }
                    }
                }
                RawInline::Span { attrs, children } => {
                    let resolved = self.resolve(Some(attrs), Kind::Span);
                    if resolved.dropped {
                        continue;
                    }
                    self.note_step(resolved.step);
                    let children = self.convert(children);
                    out.push(Inline::Span { style: resolved.style, step: resolved.step, behavior: resolved.behavior, source_attrs: attrs.raw.clone(), children });
                }
                RawInline::Image { alt, src, span, attrs } => {
                    let resolved = self.resolve(attrs.as_ref(), Kind::Span);
                    if src.contains(DYNAMIC_MARK) {
                        self.dynamic_here(*span);
                        continue;
                    }
                    match style::image_source(src) {
                        Ok(src) => {
                            if matches!(src, ImageSource::Url { .. }) {
                                self.forbid_in_lesson("external images", *span);
                            }
                            out.push(Inline::Image { src, alt: alt.clone(), style: resolved.style, source_attrs: attrs.as_ref().map(|attrs| attrs.raw.clone()).unwrap_or_default() });
                        }
                        Err(invalid) => {
                            self.report(*span, invalid);
                            out.push(Inline::Text { value: alt.clone() });
                        }
                    }
                }
            }
        }
        out
    }

    fn blocks(&mut self, nodes: &[Node]) -> Vec<Block> {
        let mut out = Vec::new();
        let mut chain: Option<Chain> = None;
        for node in nodes {
            if let Node::Directive(directive) = node {
                match directive.name.value.as_str() {
                    "if" => {
                        chain = self.if_branch(directive, &mut out);
                        continue;
                    }
                    "else" => {
                        match chain.take() {
                            Some(previous) => chain = self.else_branch(directive, previous, &mut out),
                            None => {
                                self.diags
                                    .error("R607", directive.name.span, tr("`else` precisa vir logo depois de um bloco `if`", "`else` must come right after an `if` block"))
                                    .hint(tr("Escreva ::: if condição ... ::: e, em seguida, ::: else ... :::", "Write ::: if condition ... ::: and then ::: else ... :::"));
                            }
                        }
                        continue;
                    }
                    _ => {}
                }
            }
            chain = None;
            let mut blocks = self.node(node);
            if self.tracking
                && let [block] = blocks.as_mut_slice()
                && block.line == node_line(node) {
                    block.end_line = Some(node.end_line());
                }
            out.extend(blocks.into_iter().filter(|block| !is_dropped(block)));
        }
        out
    }

    fn branch_children(&mut self, directive: &Directive, when: Option<Expr>, out: &mut Vec<Block>) {
        if !directive.container {
            self.diags
                .error("R104", directive.name.span, tr(format!("`{}` precisa de conteúdo: use `::: {}` e feche com `:::`", directive.name.value, directive.name.value), format!("`{}` needs content: use `::: {}` and close it with `:::`", directive.name.value, directive.name.value)));
            return;
        }
        let mut blocks = self.blocks(&directive.children);
        if let Some(when) = when {
            for block in &mut blocks {
                block.behavior.when = Some(and(block.behavior.when.take(), when.clone()));
            }
        }
        out.extend(blocks);
    }

    fn if_branch(&mut self, directive: &Directive, out: &mut Vec<Block>) -> Option<Chain> {
        let Some(value) = directive.value.clone() else {
            self.diags.error("R603", directive.name.span, tr("`if` precisa de uma condição, como ::: if $detalhes", "`if` needs a condition, such as ::: if $details"));
            return None;
        };
        match self.condition(&value)? {
            Folded::Static(Value::Bool(true)) => {
                self.branch_children(directive, None, out);
                Some(Chain::Done)
            }
            Folded::Dynamic(expr, _) => {
                self.branch_children(directive, Some(expr.clone()), out);
                Some(Chain::Open(vec![expr]))
            }
            _ => Some(Chain::Open(Vec::new())),
        }
    }

    fn else_branch(&mut self, directive: &Directive, chain: Chain, out: &mut Vec<Block>) -> Option<Chain> {
        let condition = match &directive.value {
            None => None,
            Some(value) => match value.value.strip_prefix("if ") {
                Some(rest) => {
                    let offset = value.value.len() - rest.trim_start().len();
                    Some(Text::new(rest.trim(), value.span.line, value.span.from + offset))
                }
                None => {
                    self.diags.error("R603", value.span, tr("depois de `else` só pode vir `if` e uma condição", "only `if` and a condition can follow `else`")).hint(tr("Exemplo: ::: else if $plano == pro", "Example: ::: else if $plan == pro"));
                    return None;
                }
            },
        };
        let Chain::Open(prior) = chain else {
            return match &condition {
                Some(condition) => {
                    self.condition(condition);
                    Some(Chain::Done)
                }
                None => None,
            };
        };
        let guard = prior.iter().cloned().reduce(|a, b| Expr::Binary { operator: BinaryOp::Or, left: Box::new(a), right: Box::new(b) }).map(|any| Expr::Not { arg: Box::new(any) });
        let Some(condition) = condition else {
            self.branch_children(directive, guard, out);
            return None;
        };
        match self.condition(&condition)? {
            Folded::Static(Value::Bool(true)) => {
                let done = guard.is_none();
                self.branch_children(directive, guard, out);
                Some(if done { Chain::Done } else { Chain::Open(prior.into_iter().chain([Expr::Lit { value: Scalar::Bool(true) }]).collect()) })
            }
            Folded::Dynamic(expr, _) => {
                let when = and(guard, expr.clone());
                self.branch_children(directive, Some(when), out);
                Some(Chain::Open(prior.into_iter().chain([expr]).collect()))
            }
            _ => Some(Chain::Open(prior)),
        }
    }

    fn for_loop(&mut self, directive: &Directive) -> Vec<Block> {
        let usage = || tr("Exemplo: ::: for pessoa in $equipe, ::: for item, i in $lista ou ::: for i in 1..5", "Example: ::: for person in $team, ::: for item, i in $list or ::: for i in 1..5");
        let Some(value) = directive.value.clone() else {
            self.diags.error("R606", directive.name.span, tr("`for` precisa de uma variável e uma lista", "`for` needs a variable and a list")).hint(usage());
            return Vec::new();
        };
        let Some(split) = value.value.find(" in ") else {
            self.diags.error("R606", value.span, tr("`for` espera `nome in lista`", "`for` expects `name in list`")).hint(usage());
            return Vec::new();
        };
        let names: Vec<&str> = value.value[..split].split(',').map(str::trim).collect();
        let valid = |name: &&str| path_at(name, 0).is_some_and(|(path, end)| path.len() == 1 && end == name.len());
        if names.is_empty() || names.len() > 2 || !names.iter().all(valid) {
            self.diags.error("R606", value.span, tr("nomes inválidos no `for`", "invalid names in `for`")).hint(usage());
            return Vec::new();
        }
        let list_text = Text::new(value.value[split + 4..].trim(), value.span.line, value.span.from + split + 4 + (value.value[split + 4..].len() - value.value[split + 4..].trim_start().len()));
        let items = match self.expression(&list_text) {
            Some(Folded::Static(Value::List(items))) => items,
            Some(Folded::Dynamic(Expr::State { name }, ty @ (Type::List | Type::Records))) => {
                if !directive.container {
                    self.diags.error("R104", directive.name.span, tr("`for` precisa de conteúdo: use `::: for` e feche com `:::`", "`for` needs content: use `::: for` and close it with `:::`"));
                    return Vec::new();
                }
                return self.runtime_for(directive, &names, name, ty);
            }
            Some(Folded::Dynamic(..)) => {
                self.dynamic_here(list_text.span);
                return Vec::new();
            }
            Some(Folded::Static(other)) => {
                self.diags.error("R606", list_text.span, tr(format!("`for` precisa de uma lista, mas o valor é {}", other.ty().label()), format!("`for` needs a list, but the value is {}", other.ty().label())));
                return Vec::new();
            }
            None => return Vec::new(),
        };
        if !directive.container {
            self.diags.error("R104", directive.name.span, tr("`for` precisa de conteúdo: use `::: for` e feche com `:::`", "`for` needs content: use `::: for` and close it with `:::`"));
            return Vec::new();
        }
        let mut out = Vec::new();
        for (index, item) in items.into_iter().enumerate() {
            self.iterations += 1;
            if self.iterations > MAX_ITEMS {
                self.diags.error("R606", directive.name.span, tr(format!("repetições demais no documento (limite: {MAX_ITEMS})"), format!("too many repetitions in the document (limit: {MAX_ITEMS})")));
                break;
            }
            let mut frame = Frame::default();
            frame.names.insert(names[0].to_string(), Binding::Value(item));
            if let Some(position) = names.get(1) {
                frame.names.insert(position.to_string(), Binding::Value(Value::Number(index as f64 + 1.0)));
            }
            self.frames.push(frame);
            let blocks = self.blocks(&directive.children);
            self.frames.pop();
            out.extend(blocks);
        }
        merge_lists(out)
    }

    fn node(&mut self, node: &Node) -> Vec<Block> {
        match node {
            Node::Heading { level, text, attrs, line } => {
                let resolved = self.resolve(attrs.as_ref(), Kind::Text);
                let (content, changed) = self.inline_lines(std::slice::from_ref(text));
                let mut block = self.make(BlockKind::Heading { level: *level, content }, resolved, *line);
                if !changed {
                    block.source = self.source_of(std::slice::from_ref(text));
                }
                vec![block]
            }
            Node::Paragraph { lines, attrs, line } => {
                let resolved = self.resolve(attrs.as_ref(), Kind::Text);
                let (content, changed) = self.inline_lines(lines);
                let mut block = self.make(BlockKind::Paragraph { content }, resolved, *line);
                if !changed {
                    block.source = self.source_of(lines);
                }
                vec![block]
            }
            Node::List(list) => vec![self.list(list)],
            Node::Quote { children, line, .. } => {
                let tracking = std::mem::replace(&mut self.tracking, false);
                let children = self.blocks(children);
                self.tracking = tracking;
                vec![self.make(BlockKind::Quote { children }, Resolved::default(), *line)]
            }
            Node::Code { lang, code, attrs, line, .. } => {
                let resolved = self.resolve(attrs.as_ref(), Kind::Text);
                vec![self.make(BlockKind::Code { lang: lang.clone(), code: code.clone(), lines: highlight(code, lang.as_deref()) }, resolved, *line)]
            }
            Node::Table(table) => vec![self.table(table)],
            Node::Image { alt, src, attrs, line } => {
                let resolved = self.resolve(attrs.as_ref(), Kind::Image);
                let fit = match resolved.extra("fit").and_then(|pair| pair.value.as_ref()) {
                    Some(value) if value.value == "cover" => ImageFit::Cover,
                    Some(value) if value.value != "contain" => {
                        self.diags.error("R301", value.span, tr(format!("ajuste de imagem inválido: `{}`", value.value), format!("invalid image fit: `{}`", value.value))).hint(tr("Use cover ou contain", "Use cover or contain"));
                        ImageFit::Contain
                    }
                    _ => ImageFit::Contain,
                };
                let alt = self.interpolated(alt, false);
                let src = self.interpolated(src, false);
                match style::image_source(&src.value) {
                    Ok(source) => {
                        if matches!(source, ImageSource::Url { .. }) {
                            self.forbid_in_lesson("external images", src.span);
                        }
                        vec![self.make(BlockKind::Image { src: source, alt: alt.value.clone(), fit }, resolved, *line)]
                    }
                    Err(invalid) => {
                        self.report(src.span, invalid);
                        Vec::new()
                    }
                }
            }
            Node::Directive(directive) => self.directive(directive),
            Node::Placeholder { name, span } => self.placeholder(name, *span),
        }
    }

    fn placeholder(&mut self, name: &str, span: Span) -> Vec<Block> {
        let in_component = self.barrier_frame().is_some_and(|frame| frame.barrier == Some(Barrier::Component));
        if name == "content" {
            if let Some(frame) = self.barrier_frame()
                && frame.barrier == Some(Barrier::Component) {
                    frame.used_content = true;
                    return frame.content.clone();
                }
            if let Some(frame) = self.barrier_frame() {
                frame.used_content = true;
                return Vec::new();
            }
            self.diags.error("R401", span, tr("`$content` só pode ser usado dentro de um @component", "`$content` can only be used inside a @component"));
            return Vec::new();
        }
        if self.lookup(name).is_some() {
            let text = Text { value: format!("${name}"), span };
            let (content, _) = self.inline_lines(std::slice::from_ref(&text));
            return vec![self.make(BlockKind::Paragraph { content }, Resolved::default(), span.line)];
        }
        if let Some(frame) = self.barrier_frame() {
            frame.slot_names.insert(name.to_string());
            if in_component {
                return frame.slots.get(name).cloned().unwrap_or_default();
            }
            return Vec::new();
        }
        let content = self.inline(&Text { value: format!("${name}"), span });
        vec![self.make(BlockKind::Paragraph { content }, Resolved::default(), span.line)]
    }

    fn list(&mut self, list: &List) -> Block {
        let mut items = Vec::new();
        for item in &list.items {
            let resolved = self.resolve(item.attrs.as_ref(), Kind::Text);
            if resolved.dropped {
                continue;
            }
            self.note_step(resolved.step);
            let (content, changed) = self.inline_lines(&item.lines);
            let children = item.children.iter().map(|nested| self.list(nested)).collect();
            let source = if changed { None } else { self.source_of(&item.lines) };
            let id = self.register_anchor(resolved.id.as_ref());
            items.push(ListItem { content, children, id, style: resolved.style, step: resolved.step, behavior: resolved.behavior, line: item.line, source });
        }
        Block { kind: BlockKind::List { ordered: list.ordered, start: list.start, items }, id: None, style: Style::default(), step: None, behavior: Behavior::default(), line: list.line, end_line: None, source: None }
    }

    fn table(&mut self, table: &Table) -> Block {
        let header = table.header.iter().map(|cell| self.inline_lines(std::slice::from_ref(cell)).0).collect();
        let rows = table.rows.iter().map(|row| row.iter().map(|cell| self.inline_lines(std::slice::from_ref(cell)).0).collect()).collect();
        Block {
            kind: BlockKind::Table { header, aligns: table.aligns.clone(), rows },
            id: None,
            style: Style::default(),
            step: None,
            behavior: Behavior::default(),
            line: table.line,
            end_line: None,
            source: None,
        }
    }

    fn directive(&mut self, directive: &Directive) -> Vec<Block> {
        let name = directive.name.value.as_str();
        if name.is_empty() {
            return Vec::new();
        }
        if name == "for" {
            return self.for_loop(directive);
        }
        let interpolated;
        let directive = if matches!(name, "icon" | "spacer") && directive.value.as_ref().is_some_and(|value| value.value.contains('$')) {
            let mut copy = directive.clone();
            copy.value = copy.value.as_ref().map(|value| self.interpolated(value, false));
            interpolated = copy;
            &interpolated
        } else {
            directive
        };
        let leaf_only = matches!(name, "icon" | "spacer" | "divider" | "input" | "sound");
        let takes_value = matches!(name, "icon" | "spacer" | "sound");
        if leaf_only && !directive.children.is_empty() {
            self.diags
                .warning("R109", directive.name.span, tr(format!("`{name}` não aceita conteúdo; use `:: {name}` em uma linha só"), format!("`{name}` does not accept content; use `:: {name}` on a single line")));
        }
        if !takes_value && BUILTIN_BLOCKS.contains(&name)
            && let Some(value) = &directive.value {
                self.diags.warning("R110", value.span, tr(format!("o texto `{}` depois de `{name}` será ignorado", value.value), format!("the text `{}` after `{name}` will be ignored", value.value))).hint(tr("Use atributos entre chaves: {chave=valor}", "Use attributes in braces: {key=value}"));
            }
        match name {
            "stack" => vec![self.layout(directive, LayoutKind::Stack)],
            "row" => vec![self.layout(directive, LayoutKind::Row)],
            "columns" => vec![self.layout(directive, LayoutKind::Columns)],
            "grid" => vec![self.layout(directive, LayoutKind::Grid)],
            "box" => vec![self.layout(directive, LayoutKind::Box)],
            "steps" => vec![self.steps(directive)],
            "notes" => {
                if let Some(raw) = &directive.raw {
                    let text = self.interpolate(&Text::new(raw.clone(), directive.line, 0), false).0;
                    self.notes.push(text);
                }
                Vec::new()
            }
            "chart" => self.chart(directive).into_iter().collect(),
            "table" => self.data_table(directive).into_iter().collect(),
            "slot" => {
                self.diags
                    .error("R416", directive.name.span, tr("`::: slot` só pode ser usado dentro de um componente", "`::: slot` can only be used inside a component"))
                    .hint(tr("Use-o para preencher um espaço nomeado: ::: meu-componente ... ::: slot nome ... ::: ... :::", "Use it to fill a named slot: ::: my-component ... ::: slot name ... ::: ... :::"));
                Vec::new()
            }
            "icon" => self.icon(directive).into_iter().collect(),
            "input" => self.input(directive).into_iter().collect(),
            "sound" => self.sound(directive).into_iter().collect(),
            "spacer" => {
                let size = match &directive.value {
                    Some(value) => match style::space(&value.value) {
                        Ok(size) => size,
                        Err(invalid) => {
                            self.report(value.span, invalid);
                            32.0
                        }
                    },
                    None => 32.0,
                };
                let resolved = self.resolve(directive.attrs.as_ref(), Kind::Text);
                vec![self.make(BlockKind::Spacer { size }, resolved, directive.line)]
            }
            "divider" => {
                let resolved = self.resolve(directive.attrs.as_ref(), Kind::Text);
                vec![self.make(BlockKind::Divider, resolved, directive.line)]
            }
            _ if self.components.contains_key(name) => self.expand(directive),
            _ => {
                let candidates: Vec<String> = BUILTIN_BLOCKS.iter().map(|n| n.to_string()).chain(self.components.keys().cloned()).collect();
                let pending = self.diags.error("R108", directive.name.span, tr(format!("bloco desconhecido: `{name}`"), format!("unknown block: `{name}`")));
                match suggest(name, candidates.iter().map(String::as_str)) {
                    Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                    None => pending.hint(tr(format!("Blocos disponíveis: {}", candidates.join(", ")), format!("Available blocks: {}", candidates.join(", ")))),
                };
                Vec::new()
            }
        }
    }

    fn layout(&mut self, directive: &Directive, layout: LayoutKind) -> Block {
        let resolved = self.resolve(directive.attrs.as_ref(), Kind::Layout(layout));
        let children = self.blocks(&directive.children);
        let mut cols = None;
        let mut widths = None;
        let mut wrap = false;
        for pair in resolved.extra.clone() {
            let value = pair.value.as_ref();
            match pair.key.value.as_str() {
                "cols" => match value.and_then(|v| v.value.parse::<u32>().ok()).filter(|n| (1..=12).contains(n)) {
                    Some(n) => cols = Some(n),
                    None => {
                        self.diags.error("R301", value.map_or(pair.key.span, |v| v.span), tr("número de colunas inválido", "invalid number of columns")).hint(tr("Use um número de 1 a 12, como cols=3", "Use a number from 1 to 12, such as cols=3"));
                    }
                },
                "widths" => {
                    let parsed: Option<Vec<f32>> = value.and_then(|v| v.value.split_whitespace().map(|part| part.parse::<f32>().ok().filter(|n| *n > 0.0)).collect::<Option<Vec<_>>>());
                    match parsed {
                        Some(list) if !list.is_empty() => {
                            if list.len() != children.len() {
                                self.diags.warning(
                                    "R302",
                                    value.map_or(pair.key.span, |v| v.span),
                                    tr(format!("`widths` tem {} valores, mas há {} colunas", list.len(), children.len()), format!("`widths` has {} values, but there are {} columns", list.len(), children.len())),
                                );
                            }
                            widths = Some(list);
                        }
                        _ => {
                            self.diags
                                .error("R301", value.map_or(pair.key.span, |v| v.span), tr("proporções de colunas inválidas", "invalid column proportions"))
                                .hint(tr("Use números positivos separados por espaço, como widths=\"2 1\"", "Use positive numbers separated by spaces, such as widths=\"2 1\""));
                        }
                    }
                }
                "wrap" => match style::flag(value.map(|v| v.value.as_str())) {
                    Ok(flag) => wrap = flag,
                    Err(invalid) => self.report(pair.key.span, invalid),
                },
                _ => {}
            }
        }
        if layout == LayoutKind::Grid && cols.is_none() {
            cols = Some(children.len().clamp(1, 3) as u32);
        }
        self.make(BlockKind::Layout { layout, cols, widths, wrap, children }, resolved, directive.line)
    }

    fn steps(&mut self, directive: &Directive) -> Block {
        let resolved = self.resolve(directive.attrs.as_ref(), Kind::Layout(LayoutKind::Stack));
        let mut children = self.blocks(&directive.children);
        for child in &mut children {
            if let BlockKind::List { items, .. } = &mut child.kind {
                for item in items {
                    self.step += 1;
                    item.step = Some(self.step);
                }
            } else {
                self.step += 1;
                child.step = Some(self.step);
            }
        }
        self.max_step = self.max_step.max(self.step);
        self.make(BlockKind::Layout { layout: LayoutKind::Stack, cols: None, widths: None, wrap: false, children }, resolved, directive.line)
    }

    fn icon(&mut self, directive: &Directive) -> Option<Block> {
        let Some(value) = &directive.value else {
            self.diags.error("R114", directive.name.span, tr("informe o nome do ícone", "provide the icon name")).hint(tr("Exemplo: :: icon rocket {size=3xl color=accent}", "Example: :: icon rocket {size=3xl color=accent}"));
            return None;
        };
        if !icons::is_icon(&value.value) {
            let pending = self.diags.error("R115", value.span, tr(format!("ícone desconhecido: `{}`", value.value), format!("unknown icon: `{}`", value.value)));
            match suggest(&value.value, icons::names()) {
                Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                None => pending.hint(tr("Veja a lista de ícones na documentação da linguagem", "See the icon list in the language documentation")),
            };
            return None;
        }
        let resolved = self.resolve(directive.attrs.as_ref(), Kind::Text);
        Some(self.make(BlockKind::Icon { name: value.value.clone() }, resolved, directive.line))
    }

    pub fn forbid_external_backgrounds_in_lesson(&mut self, preamble: &Preamble, pages: &[Page]) {
        if self.profile != Profile::Lesson {
            return;
        }
        for (_, attrs) in &preamble.styles {
            for pair in attrs.pairs.iter().filter(|pair| pair.key.value == "image") {
                if let Some(value) = pair.value.as_ref().filter(|value| is_external_url(&value.value)) {
                    self.forbid_in_lesson("external images", value.span);
                }
            }
        }
        let mut lines = Vec::new();
        for page in pages {
            let document = serde_json::to_value(page).unwrap_or_default();
            collect_external_backgrounds(&document, page.line, &mut lines);
        }
        for line in lines {
            self.forbid_in_lesson("external images", Span::new(line, 0, 1));
        }
    }

    pub fn forbid_persist_in_lesson(&mut self, preamble: &Preamble) {
        for name in &preamble.persist {
            self.forbid_in_lesson("`@persist`", Span::new(name.span.line, name.span.from, name.span.from + name.value.len()));
        }
    }

    fn forbid_in_lesson(&mut self, what: &str, span: Span) {
        if self.profile != Profile::Lesson {
            return;
        }
        self.diags
            .error("R212", span, tr(format!("{what} não é permitido em aulas"), format!("{what} is not allowed in lessons")))
            .hint(tr("Aulas são texto: sem imagens externas, sons, campos de texto, `@persist`, `next` nem `hidden`", "Lessons are text: no external images, sounds, text inputs, `@persist`, `next` or `hidden`"));
    }

    fn sound(&mut self, directive: &Directive) -> Option<Block> {
        self.forbid_in_lesson("`sound`", directive.name.span);
        let resolved = self.resolve(directive.attrs.as_ref(), Kind::Sound);
        let Some(value) = &directive.value else {
            self.diags
                .error("R117", directive.name.span, tr("`sound` precisa do endereço do arquivo de áudio", "`sound` needs the address of the audio file"))
                .hint(tr("Exemplo: :: sound https://exemplo.com/porta.mp3 {name=porta}", "Example: :: sound https://example.com/door.mp3 {name=door}"));
            return None;
        };
        let src = match style::audio_source(&value.value) {
            Ok(src) => src,
            Err(invalid) => {
                self.report(value.span, invalid);
                return None;
            }
        };
        let name = resolved.extra("name").cloned().and_then(|pair| {
            let name = pair.value.as_ref().map(|value| value.value.clone()).filter(|name| !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')));
            match &name {
                None => {
                    self.diags.error("R301", pair.value.as_ref().map_or(pair.key.span, |value| value.span), tr("nome de som inválido", "invalid sound name")).hint(tr("Use letras, números e hífens, como name=porta", "Use letters, digits and hyphens, such as name=door"));
                }
                Some(name) => {
                    let span = pair.value.as_ref().map_or(pair.key.span, |value| value.span);
                    if self.sounds.insert(name.clone(), span).is_some() {
                        self.diags.error("R616", span, tr(format!("já existe outro som `{name}` nesta página"), format!("another sound `{name}` already exists on this page")));
                    }
                }
            }
            name
        });
        let flag = |lowerer: &mut Self, key: &str| resolved.extra(key).cloned().map(|pair| lowerer.switch(&pair, key).unwrap_or(false)).unwrap_or(false);
        let (autoplay, repeat) = (flag(self, "autoplay"), flag(self, "loop"));
        let volume = resolved.extra("volume").cloned().and_then(|pair| {
            let parsed = pair.value.as_ref().and_then(|value| value.value.trim_end_matches('%').parse::<f64>().ok().map(|n| if value.value.ends_with('%') { n / 100.0 } else { n })).filter(|n| (0.0..=1.0).contains(n));
            if parsed.is_none() {
                self.diags.error("R301", pair.value.as_ref().map_or(pair.key.span, |value| value.span), tr("volume inválido", "invalid volume")).hint(tr("Use um número de 0 a 1 ou uma porcentagem, como volume=0.5 ou volume=50%", "Use a number from 0 to 1 or a percentage, such as volume=0.5 or volume=50%"));
            }
            parsed
        });
        Some(self.make(BlockKind::Sound { name, src, autoplay, repeat, volume }, resolved, directive.line))
    }

    fn check_sounds(&mut self) {
        for (name, span) in std::mem::take(&mut self.sound_refs) {
            if self.sounds.contains_key(&name) {
                continue;
            }
            let names: Vec<String> = self.sounds.keys().cloned().collect();
            let pending = self.diags.error("R615", span, tr(format!("esta página não tem o som `{name}`"), format!("this page has no sound `{name}`")));
            match suggest(&name, names.iter().map(String::as_str)) {
                Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                None => pending.hint(tr(format!("Declare o som na página: :: sound https://... {{name={name}}}"), format!("Declare the sound on the page: :: sound https://... {{name={name}}}"))),
            };
        }
    }

    fn input(&mut self, directive: &Directive) -> Option<Block> {
        self.forbid_in_lesson("`input`", directive.name.span);
        let resolved = self.resolve(directive.attrs.as_ref(), Kind::Input);
        let Some(bind) = resolved.extra("bind").cloned() else {
            self.diags
                .error("R116", directive.name.span, tr("`input` precisa de `bind` com a variável que guarda o texto", "`input` needs `bind` with the variable that holds the text"))
                .hint(tr("Declare @state nome = \"\" e use :: input {bind=$nome}", "Declare @state name = \"\" and use :: input {bind=$name}"));
            return None;
        };
        let target = bind.value.as_ref().map(|value| value.value.trim().to_string()).unwrap_or_default();
        let span = bind.value.as_ref().map_or(bind.key.span, |value| value.span);
        let Some(name) = target.strip_prefix('$').filter(|name| !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))) else {
            self.diags.error("R116", span, tr("`bind` espera uma variável, como bind=$nome", "`bind` expects a variable, such as bind=$name"));
            return None;
        };
        let (real, numeric) = match self.state_binding(name) {
            Some((real, Type::Text)) => (real, false),
            Some((real, Type::Number)) => (real, true),
            Some((_, other)) => {
                self.diags.error(
                    "R602",
                    span,
                    tr(format!("`input` liga a um @state de texto ou número, mas `${name}` guarda {}", other.label()), format!("`input` binds to a text or number @state, but `${name}` holds {}", other.label())),
                );
                return None;
            }
            None => {
                self.state_target(name, None, "", span);
                return None;
            }
        };
        self.assigned.insert(real.clone(), None);
        let placeholder = resolved.extra("placeholder").and_then(|pair| pair.value.as_ref()).map(|value| value.value.clone());
        let on_submit = match resolved.extra("on-submit").cloned() {
            Some(pair) => match &pair.value {
                Some(value) => self.actions(value),
                None => {
                    self.diags.error("R203", pair.key.span, tr("`on-submit` precisa de uma ação, como on-submit=\"go #certo\"", "`on-submit` needs an action, such as on-submit=\"go #right\""));
                    Vec::new()
                }
            },
            None => Vec::new(),
        };
        Some(self.make(BlockKind::Input { name: real, numeric, placeholder, on_submit }, resolved, directive.line))
    }

    fn records(&mut self, resolved: &Resolved, block: &str) -> Option<(Vec<Record>, Span)> {
        let pair = resolved.extra("data")?.clone();
        let Some(value) = &pair.value else {
            self.diags.error("R203", pair.key.span, tr("`data` precisa de uma lista, como data=$vendas", "`data` needs a list, such as data=$sales"));
            return None;
        };
        let items = match self.expression(value)? {
            Folded::Static(Value::List(items)) => items,
            Folded::Dynamic(..) => {
                self.dynamic_here(value.span);
                return None;
            }
            Folded::Static(other) => {
                self.diags.error("R602", value.span, tr(format!("`{block}` precisa de uma lista de registros (@data), mas o valor é {}", other.ty().label()), format!("`{block}` needs a list of records (@data), but the value is {}", other.ty().label())));
                return None;
            }
        };
        let mut records = Vec::new();
        for item in items {
            match item {
                Value::Record(record) => records.push(record),
                other => {
                    self.diags.error("R602", value.span, tr(format!("`{block}` precisa de uma lista de registros (@data), mas um item é {}", other.ty().label()), format!("`{block}` needs a list of records (@data), but an item is {}", other.ty().label())));
                    return None;
                }
            }
        }
        Some((records, value.span))
    }

    fn fields(&mut self, resolved: &Resolved, key: &str, records: &[Record]) -> Option<Vec<String>> {
        let pair = resolved.extra(key)?;
        let value = pair.value.clone()?;
        let available: BTreeSet<String> = records.iter().flat_map(|record| record.keys().cloned()).collect();
        let mut fields = Vec::new();
        for field in value.value.split_whitespace() {
            if available.contains(field) {
                fields.push(field.to_string());
            } else {
                let pending = self.diags.error("R601", value.span, tr(format!("os dados não têm o campo `{field}`"), format!("the data has no field `{field}`")));
                match suggest(field, available.iter().map(String::as_str)) {
                    Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                    None => pending.hint(tr(format!("Campos disponíveis: {}", available.iter().cloned().collect::<Vec<_>>().join(", ")), format!("Available fields: {}", available.iter().cloned().collect::<Vec<_>>().join(", ")))),
                };
            }
        }
        Some(fields)
    }

    fn data_table(&mut self, directive: &Directive) -> Option<Block> {
        let resolved = self.resolve(directive.attrs.as_ref(), Kind::Table);
        if resolved.extra("data").is_none() {
            self.diags.error("R111", directive.name.span, tr("`table` precisa de dados: :: table {data=$lista}", "`table` needs data: :: table {data=$list}")).hint(tr("Para uma tabela escrita à mão, use a sintaxe Markdown | A | B |", "For a hand-written table, use the Markdown syntax | A | B |"));
            return None;
        }
        let (records, _) = self.records(&resolved, "table")?;
        let columns = self.fields(&resolved, "columns", &records).unwrap_or_else(|| records.first().map(|record| record.keys().cloned().collect()).unwrap_or_default());
        let header = columns.iter().map(|column| vec![Inline::Text { value: column.clone() }]).collect();
        let aligns = columns
            .iter()
            .map(|column| if records.iter().all(|record| matches!(record.get(column), Some(Value::Number(_)))) { crate::block::ColumnAlign::Right } else { crate::block::ColumnAlign::Left })
            .collect();
        let rows = records
            .iter()
            .map(|record| columns.iter().map(|column| self.inline(&Text::new(record.get(column).and_then(Value::display).unwrap_or_default(), directive.line, 0))).collect())
            .collect();
        Some(self.make(BlockKind::Table { header, aligns, rows }, resolved, directive.line))
    }

    fn chart(&mut self, directive: &Directive) -> Option<Block> {
        let resolved = self.resolve(directive.attrs.as_ref(), Kind::Chart);
        let mut chart = ChartKind::Bar;
        let mut legend = None;
        let mut stacked = false;
        let mut values = false;
        let mut colors: Vec<String> = Vec::new();
        for pair in &resolved.extra {
            let value = pair.value.as_ref().map(|v| v.value.as_str());
            let span = pair.value.as_ref().map_or(pair.key.span, |v| v.span);
            match pair.key.value.as_str() {
                "type" => match value {
                    Some("bar") => chart = ChartKind::Bar,
                    Some("line") => chart = ChartKind::Line,
                    Some("area") => chart = ChartKind::Area,
                    Some("pie") => chart = ChartKind::Pie,
                    Some("donut") => chart = ChartKind::Donut,
                    _ => {
                        self.diags.error("R301", span, tr("tipo de gráfico inválido", "invalid chart type")).hint(tr("Use bar, line, area, pie ou donut", "Use bar, line, area, pie or donut"));
                    }
                },
                "legend" | "stacked" | "values" => match style::flag(value) {
                    Ok(flag) => match pair.key.value.as_str() {
                        "legend" => legend = Some(flag),
                        "stacked" => stacked = flag,
                        _ => values = flag,
                    },
                    Err(invalid) => self.report(span, invalid),
                },
                "colors" => {
                    for part in value.unwrap_or("").split_whitespace() {
                        match style::color(part, &self.theme) {
                            Ok(color) => colors.push(color),
                            Err(invalid) => self.report(span, invalid),
                        }
                    }
                }
                _ => {}
            }
        }
        let palette = self.theme_ir().palette;
        let color_for = |index: usize| colors.get(index).cloned().or_else(|| palette.get(index % palette.len().max(1)).cloned()).unwrap_or_else(|| "#888888".into());
        let labels: Vec<String>;
        let mut series = Vec::new();
        if resolved.extra("data").is_some() {
            let (records, span) = self.records(&resolved, "chart")?;
            let keys: Vec<String> = records.first().map(|record| record.keys().cloned().collect()).unwrap_or_default();
            let label = match self.fields(&resolved, "label", &records) {
                Some(fields) => fields.into_iter().next(),
                None => keys.iter().find(|key| records.iter().any(|record| !matches!(record.get(key), Some(Value::Number(_))))).cloned(),
            };
            let columns = self.fields(&resolved, "series", &records).unwrap_or_else(|| {
                keys.iter().filter(|key| Some(*key) != label.as_ref() && records.iter().all(|record| matches!(record.get(key), Some(Value::Number(_))))).cloned().collect()
            });
            labels = records.iter().map(|record| label.as_ref().and_then(|label| record.get(label)).and_then(Value::display).unwrap_or_default()).collect();
            for (index, column) in columns.iter().enumerate() {
                let mut numbers = Vec::new();
                for record in &records {
                    match record.get(column) {
                        Some(Value::Number(number)) => numbers.push(*number),
                        other => {
                            let shown = other.and_then(Value::display).unwrap_or_default();
                            self.diags.error("R112", span, tr(format!("`{shown}` em `{column}` não é um número"), format!("`{shown}` in `{column}` is not a number")));
                            numbers.push(0.0);
                        }
                    }
                }
                series.push(Series { name: column.clone(), color: color_for(index), values: numbers });
            }
        } else {
            let Some(table) = directive.children.iter().find_map(|node| match node {
                Node::Table(table) => Some(table),
                _ => None,
            }) else {
                self.diags
                    .error("R111", directive.name.span, tr("o gráfico precisa de uma tabela com os dados", "the chart needs a table with the data"))
                    .hint(tr("Coloque dentro do bloco uma tabela, ou use dados: :: chart {data=$vendas}", "Put a table inside the block, or use data: :: chart {data=$sales}"));
                return None;
            };
            let table = Table {
                header: table.header.iter().map(|cell| self.interpolated(cell, false)).collect(),
                aligns: table.aligns.clone(),
                rows: table.rows.iter().map(|row| row.iter().map(|cell| self.interpolated(cell, false)).collect()).collect(),
                line: table.line,
                end: table.end,
            };
            labels = table.rows.iter().map(|row| row.first().map(|cell| cell.value.clone()).unwrap_or_default()).collect();
            for (column, head) in table.header.iter().enumerate().skip(1) {
                let mut numbers = Vec::new();
                for row in &table.rows {
                    let value = match row.get(column) {
                        Some(cell) => match parse_number(&cell.value) {
                            Some(number) => number,
                            None => {
                                self.diags.error("R112", cell.span, tr(format!("`{}` não é um número", cell.value), format!("`{}` is not a number", cell.value))).hint(tr("Use números como 42, 3.5 ou 3,5", "Use numbers such as 42, 3.5 or 3,5"));
                                0.0
                            }
                        },
                        None => 0.0,
                    };
                    numbers.push(value);
                }
                series.push(Series { name: head.value.clone(), color: color_for(column - 1), values: numbers });
            }
        }
        if series.is_empty() {
            self.diags.error("R113", directive.name.span, tr("a tabela do gráfico precisa de pelo menos uma coluna de valores", "the chart table needs at least one column of values"));
            return None;
        }
        let legend = legend.unwrap_or(series.len() > 1 || matches!(chart, ChartKind::Pie | ChartKind::Donut));
        let scene = chart::scene(&ChartData { kind: chart, labels: &labels, series: &series, legend, stacked, values, palette: &palette });
        Some(self.make(BlockKind::Chart { chart, labels, series, legend, stacked, values, scene }, resolved, directive.line))
    }

    fn argument(&mut self, value: &Text) -> Binding {
        let trimmed = value.value.trim();
        let whole_path = trimmed.strip_prefix('$').and_then(|rest| path_at(rest, 0)).is_some_and(|(_, end)| end + 1 == trimmed.len());
        let whole_expression = trimmed.starts_with("$(") && closing_paren(trimmed, 1) == Some(trimmed.len() - 1);
        if whole_path || whole_expression {
            return match self.expression(value) {
                Some(Folded::Static(value)) => Binding::Value(value),
                Some(Folded::Dynamic(expr, ty)) => Binding::Dynamic(expr, ty),
                None => Binding::Value(Value::Text(value.value.clone())),
            };
        }
        let (text, _) = self.interpolate(value, true);
        Binding::Value(literal(&text))
    }

    fn expand(&mut self, directive: &Directive) -> Vec<Block> {
        let name = directive.name.value.clone();
        if self.expansion.contains(&name) {
            self.diags.error("R404", directive.name.span, tr(format!("o componente `{name}` usa a si mesmo"), format!("component `{name}` uses itself")));
            return Vec::new();
        }
        if self.expansion.len() >= 16 {
            self.diags.error("R405", directive.name.span, tr("componentes aninhados demais (limite: 16 níveis)", "components nested too deeply (limit: 16 levels)"));
            return Vec::new();
        }
        let definition = self.components[&name].clone();
        let declared = |key: &str| definition.params.iter().any(|(param, _)| param.value == key);
        let mut args: HashMap<String, Binding> = HashMap::new();
        let mut usage = Resolved::default();
        if let Some(attrs) = &directive.attrs {
            if !attrs.classes.is_empty() || attrs.id.is_some() || !attrs.conditional.is_empty() {
                self.diags
                    .error("R406", attrs.span, tr("componentes recebem apenas parâmetros, não .estilos nem #id", "components only accept parameters, not .styles or #id"))
                    .hint(tr(format!("Para estilizar, envolva-o: ::: box {{.estilo}} ... ::: {name} ... :::"), format!("To style it, wrap it: ::: box {{.style}} ... ::: {name} ... :::")));
            }
            for pair in &attrs.pairs {
                let key = pair.key.value.as_str();
                if declared(key) {
                    let binding = match &pair.value {
                        Some(value) => self.argument(value),
                        None => Binding::Value(Value::Bool(true)),
                    };
                    if let Some(value) = &pair.value
                        && let Some(options) = definition.params.iter().find(|(param, _)| param.value == key).and_then(|(_, default)| default.as_ref()).and_then(|default| options_of(&default.value))
                        && let Binding::Value(Value::Text(given)) = &binding
                        && !options.contains(&given.as_str()) {
                            let fallback = Binding::Value(Value::Text(options[0].to_string()));
                            let pending = self.diags.error("R418", value.span, tr(format!("valor inválido para `{key}`: `{given}`"), format!("invalid value for `{key}`: `{given}`")));
                            match suggest(given, options.iter().copied()) {
                                Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                                None => pending.hint(tr(format!("Valores aceitos: {}", options.join(", ")), format!("Accepted values: {}", options.join(", ")))),
                            };
                            args.insert(key.to_string(), fallback);
                            continue;
                        }
                    args.insert(key.to_string(), binding);
                } else if matches!(key, "step" | "if" | "click" | "key") {
                    self.apply(pair, Kind::Text, &mut usage, false);
                } else {
                    let params: Vec<&str> = definition.params.iter().map(|(param, _)| param.value.as_str()).collect();
                    let pending = self.diags.error("R407", pair.key.span, tr(format!("o componente `{name}` não tem o parâmetro `{key}`"), format!("component `{name}` has no parameter `{key}`")));
                    match suggest(key, params.iter().copied()) {
                        Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                        None => pending.hint(tr(format!("Parâmetros de `{name}`: {}", if params.is_empty() { tr("nenhum", "none").to_string() } else { params.join(", ") }), format!("Parameters of `{name}`: {}", if params.is_empty() { tr("nenhum", "none").to_string() } else { params.join(", ") }))),
                    };
                }
            }
        }
        self.check_key(&usage);
        if usage.dropped {
            return Vec::new();
        }
        if let Some(value) = &directive.value {
            self.diags.error("R408", value.span, tr("componentes recebem parâmetros nomeados entre chaves", "components take named parameters in braces")).hint(tr(format!("Exemplo: ::: {name} {{parametro=valor}}"), format!("Example: ::: {name} {{parameter=value}}")));
        }
        for (param, default) in &definition.params {
            if args.contains_key(&param.value) {
                continue;
            }
            match default {
                Some(default) => {
                    let binding = match options_of(&default.value) {
                        Some(options) => Binding::Value(Value::Text(options[0].to_string())),
                        None => self.argument(default),
                    };
                    args.insert(param.value.clone(), binding);
                }
                None => {
                    self.diags
                        .error("R409", directive.name.span, tr(format!("falta o parâmetro obrigatório `{}` do componente `{name}`", param.value), format!("missing required parameter `{}` of component `{name}`", param.value)))
                        .hint(tr(format!("Exemplo: ::: {name} {{{}=...}}", param.value), format!("Example: ::: {name} {{{}=...}}", param.value)));
                }
            }
        }
        let mut content_nodes = Vec::new();
        let mut slot_sources: Vec<(Text, &[Node])> = Vec::new();
        for child in &directive.children {
            match child {
                Node::Directive(slot) if slot.name.value == "slot" && slot.container => match &slot.value {
                    Some(slot_name) if slot_sources.iter().any(|(existing, _)| existing.value == slot_name.value) => {
                        self.diags.error("R417", slot_name.span, tr(format!("o espaço `{}` foi preenchido mais de uma vez", slot_name.value), format!("slot `{}` was filled more than once", slot_name.value)));
                    }
                    Some(slot_name) => slot_sources.push((slot_name.clone(), &slot.children)),
                    None => {
                        self.diags.error("R415", slot.name.span, tr("informe o nome do espaço", "provide the slot name")).hint(tr("Exemplo: ::: slot lado-b", "Example: ::: slot side-b"));
                    }
                },
                other => content_nodes.push(other.clone()),
            }
        }
        let content = self.blocks(&content_nodes);
        let mut slots: HashMap<String, Vec<Block>> = HashMap::new();
        for (slot_name, nodes) in &slot_sources {
            let blocks = self.blocks(nodes);
            slots.insert(slot_name.value.clone(), blocks);
        }
        let previous = (!self.diags.has_origin()).then(|| self.diags.set_origin(Some((directive.name.span, name.clone()))));
        self.expansion.push(name.clone());
        self.frames.push(Frame { names: args, barrier: Some(Barrier::Component), content, slots, ..Frame::default() });
        let tracking = std::mem::replace(&mut self.tracking, false);
        let mut blocks = self.blocks(&definition.body);
        self.tracking = tracking;
        let frame = self.frames.pop().expect("pushed above");
        self.expansion.pop();
        if let Some(previous) = previous {
            self.diags.set_origin(previous);
        }
        let (missing, used_content, slot_names) = (frame.missing, frame.used_content, frame.slot_names);
        for (slot_name, _) in &slot_sources {
            if slot_names.contains(&slot_name.value) {
                continue;
            }
            let pending = self.diags.error("R414", slot_name.span, tr(format!("o componente `{name}` não tem o espaço `{}`", slot_name.value), format!("component `{name}` has no slot `{}`", slot_name.value)));
            match suggest(&slot_name.value, slot_names.iter().map(String::as_str)) {
                Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                None if slot_names.is_empty() => pending.hint(tr(format!("`{name}` não declara espaços nomeados; use $nome sozinho numa linha do @component"), format!("`{name}` declares no named slots; use $name alone on a line of the @component"))),
                None => pending.hint(tr(format!("Espaços de `{name}`: {}", slot_names.iter().cloned().collect::<Vec<_>>().join(", ")), format!("Slots of `{name}`: {}", slot_names.iter().cloned().collect::<Vec<_>>().join(", ")))),
            };
        }
        for param in missing {
            let local = self.frames[1..].iter().any(|frame| frame.names.contains_key(&param));
            if local {
                self.diags
                    .error("R410", directive.name.span, tr(format!("o componente `{name}` usa `${param}`, que só existe onde o componente é usado"), format!("component `{name}` uses `${param}`, which only exists where the component is used")))
                    .hint(tr(
                        format!("Componentes enxergam os próprios parâmetros e as definições do início do documento. Passe o valor: @component {name} {{{param}}} e :: {name} {{{param}=${param}}}"),
                        format!("Components see their own parameters and the definitions at the start of the document. Pass the value: @component {name} {{{param}}} and :: {name} {{{param}=${param}}}"),
                    ));
            } else {
                self.diags
                    .error("R410", directive.name.span, tr(format!("o componente `{name}` usa `${param}`, que não é um parâmetro declarado"), format!("component `{name}` uses `${param}`, which is not a declared parameter")))
                    .hint(tr(format!("Declare-o em @component {name} {{{param}}}"), format!("Declare it in @component {name} {{{param}}}")));
            }
        }
        if !used_content && !content_nodes.is_empty() {
            self.diags.warning("R411", directive.name.span, tr(format!("o componente `{name}` não usa `$content`; o conteúdo será ignorado"), format!("component `{name}` does not use `$content`; the content will be ignored")));
        }
        let step = usage.step;
        let behavior = usage.behavior;
        if blocks.len() == 1 {
            if step.is_some() {
                blocks[0].step = step;
                self.note_step(step);
            }
            merge_behavior(&mut blocks[0].behavior, behavior);
            return blocks;
        }
        let resolved = Resolved { step, behavior, ..Resolved::default() };
        vec![self.make(BlockKind::Layout { layout: LayoutKind::Stack, cols: None, widths: None, wrap: false, children: blocks }, resolved, directive.line)]
    }
}

fn split_definitions<'l, 'a>(lines: &'l [Line<'a>]) -> (Vec<Line<'a>>, &'l [Line<'a>]) {
    let mut definitions = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trimmed();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            index += 1;
        } else if trimmed.starts_with("@let ") || trimmed.starts_with("@state ") {
            definitions.push(lines[index]);
            index += 1;
        } else {
            break;
        }
    }
    if definitions.is_empty() {
        return (definitions, lines);
    }
    (definitions, &lines[index..])
}

fn closing_paren(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0;
    let mut quote: Option<u8> = None;
    let mut i = open;
    while i < bytes.len() {
        let byte = bytes[i];
        match quote {
            Some(q) => {
                if byte == b'\\' {
                    i += 1;
                } else if byte == q {
                    quote = None;
                }
            }
            None => match byte {
                b'"' | b'\'' => quote = Some(byte),
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            },
        }
        i += 1;
    }
    None
}

fn literal(text: &str) -> Value {
    match text {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    }
    let numeric = !text.is_empty()
        && text.trim_start_matches('-').chars().all(|c| c.is_ascii_digit() || c == '.')
        && text.trim_start_matches('-').starts_with(|c: char| c.is_ascii_digit());
    if numeric
        && let Ok(number) = text.parse::<f64>()
        && format_number(number) == text {
            return Value::Number(number);
        }
    Value::Text(text.to_string())
}

fn options_of(default: &str) -> Option<Vec<&str>> {
    default.contains('|').then(|| default.split('|').map(str::trim).filter(|option| !option.is_empty()).collect::<Vec<_>>()).filter(|options| options.len() > 1)
}

fn and(existing: Option<Expr>, next: Expr) -> Expr {
    match existing {
        Some(existing) => Expr::Binary { operator: BinaryOp::And, left: Box::new(existing), right: Box::new(next) },
        None => next,
    }
}

fn dropped() -> Expr {
    Expr::Lit { value: Scalar::Bool(false) }
}

fn is_dropped(block: &Block) -> bool {
    block.behavior.when == Some(dropped())
}

fn merge_behavior(target: &mut Behavior, extra: Behavior) {
    if let Some(when) = extra.when {
        target.when = Some(and(target.when.take(), when));
    }
    target.variants.extend(extra.variants);
    target.click.extend(extra.click);
    if extra.key.is_some() {
        target.key = extra.key;
    }
}

fn node_line(node: &Node) -> usize {
    match node {
        Node::Heading { line, .. } | Node::Paragraph { line, .. } | Node::Quote { line, .. } | Node::Code { line, .. } | Node::Image { line, .. } => *line,
        Node::List(list) => list.line,
        Node::Table(table) => table.line,
        Node::Directive(directive) => directive.line,
        Node::Placeholder { span, .. } => span.line,
    }
}

fn merge_lists(blocks: Vec<Block>) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::with_capacity(blocks.len());
    for block in blocks {
        let mergeable = |block: &Block| block.style == Style::default() && block.step.is_none() && block.behavior.is_empty();
        if let BlockKind::List { ordered, items, .. } = &block.kind
            && mergeable(&block)
            && let Some(previous) = out.last_mut()
            && mergeable(previous)
            && let BlockKind::List { ordered: previous_ordered, items: previous_items, .. } = &mut previous.kind
            && previous_ordered == ordered {
                previous_items.extend(items.iter().cloned());
                previous.end_line = previous.end_line.zip(block.end_line).map(|(_, end)| end);
                continue;
            }
        out.push(block);
    }
    out
}

fn field_alias(name: &str) -> Option<&'static str> {
    match name {
        "pagina" | "página" | "pag" | "num" | "numero" | "número" | "n" => Some("page"),
        "paginas" | "páginas" | "total" => Some("pages"),
        "titulo" | "título" => Some("title"),
        "autor" | "autora" => Some("author"),
        "data" | "hoje" | "today" => Some("date"),
        _ => None,
    }
}

fn push_text(out: &mut Vec<Inline>, value: &str, dynamic: &[Expr]) {
    let marker = format!("{DATE_MARK}date{DATE_MARK}");
    let mut parts = value.split(marker.as_str()).peekable();
    while let Some(part) = parts.next() {
        for (index, piece) in part.split(DYNAMIC_MARK).enumerate() {
            if index % 2 == 1 {
                if let Some(expr) = piece.parse::<usize>().ok().and_then(|index| dynamic.get(index)) {
                    out.push(Inline::Expr { expr: expr.clone() });
                }
            } else if !piece.is_empty() {
                match out.last_mut() {
                    Some(Inline::Text { value }) => value.push_str(piece),
                    _ => out.push(Inline::Text { value: piece.to_string() }),
                }
            }
        }
        if parts.peek().is_some() {
            out.push(Inline::Field { name: "date".to_string() });
        }
    }
}

fn clear_steps(block: &mut Block) -> bool {
    let mut found = block.step.take().is_some();
    match &mut block.kind {
        BlockKind::Layout { children, .. } | BlockKind::Quote { children } | BlockKind::Each { children, .. } => {
            for child in children {
                found |= clear_steps(child);
            }
        }
        BlockKind::List { items, .. } => {
            for item in items {
                found |= item.step.take().is_some();
                for nested in &mut item.children {
                    found |= clear_steps(nested);
                }
                found |= clear_inline_steps(&mut item.content);
            }
        }
        BlockKind::Heading { content, .. } | BlockKind::Paragraph { content } => found |= clear_inline_steps(content),
        _ => {}
    }
    found
}

fn clear_inline_steps(content: &mut [Inline]) -> bool {
    let mut found = false;
    for inline in content {
        match inline {
            Inline::Span { step, children, .. } => {
                found |= step.take().is_some();
                found |= clear_inline_steps(children);
            }
            Inline::Strong { children } | Inline::Emphasis { children } | Inline::Strike { children } | Inline::Link { children, .. } => {
                found |= clear_inline_steps(children);
            }
            _ => {}
        }
    }
    found
}

fn relocate(block: &mut Block) {
    block.line = 0;
    block.source = None;
    match &mut block.kind {
        BlockKind::Layout { children, .. } | BlockKind::Quote { children } | BlockKind::Each { children, .. } => children.iter_mut().for_each(relocate),
        BlockKind::List { items, .. } => {
            for item in items {
                item.line = 0;
                item.source = None;
                item.children.iter_mut().for_each(relocate);
            }
        }
        _ => {}
    }
}

fn needs<'v>(value: Option<&'v str>, key: &str) -> Parsed<&'v str> {
    value.ok_or_else(|| Invalid { message: tr(format!("o atributo `{key}` precisa de um valor"), format!("attribute `{key}` needs a value")), hint: Some(tr(format!("Exemplo: {{{key}=...}}"), format!("Example: {{{key}=...}}"))) })
}

pub fn valid_token(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn inline_plain(content: &[Inline], out: &mut String) {
    for inline in content {
        match inline {
            Inline::Text { value } | Inline::Code { value } => out.push_str(value),
            Inline::Strong { children } | Inline::Emphasis { children } | Inline::Strike { children } | Inline::Link { children, .. } | Inline::Span { children, .. } => inline_plain(children, out),
            Inline::Image { alt, .. } => out.push_str(alt),
            Inline::Expr { .. } | Inline::Field { .. } => {}
            Inline::Break => out.push(' '),
        }
    }
}

fn first_heading(blocks: &[Block]) -> Option<String> {
    for block in blocks {
        match &block.kind {
            BlockKind::Heading { content, .. } => {
                let mut title = String::new();
                inline_plain(content, &mut title);
                let title = title.trim().to_string();
                if !title.is_empty() {
                    return Some(title);
                }
            }
            BlockKind::Layout { children, .. } => {
                if let Some(title) = first_heading(children) {
                    return Some(title);
                }
            }
            _ => {}
        }
    }
    None
}

fn shortcut(value: &str) -> Option<String> {
    let value = value.trim();
    let named = match value.to_ascii_lowercase().as_str() {
        "up" | "arrowup" => Some("ArrowUp"),
        "down" | "arrowdown" => Some("ArrowDown"),
        "left" | "arrowleft" => Some("ArrowLeft"),
        "right" | "arrowright" => Some("ArrowRight"),
        "enter" => Some("Enter"),
        "space" => Some("Space"),
        _ => None,
    };
    if let Some(named) = named {
        return Some(named.to_string());
    }
    let mut chars = value.chars();
    match (chars.next(), chars.next()) {
        (Some(char), None) if char.is_ascii_alphanumeric() => Some(char.to_ascii_lowercase().to_string()),
        _ => None,
    }
}

fn is_external_url(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

fn collect_external_backgrounds(value: &serde_json::Value, line: usize, lines: &mut Vec<usize>) {
    match value {
        serde_json::Value::Array(items) => items.iter().for_each(|item| collect_external_backgrounds(item, line, lines)),
        serde_json::Value::Object(map) => {
            let line = map.get("line").and_then(serde_json::Value::as_u64).map_or(line, |found| found as usize);
            let external = map.get("bgImage").is_some_and(|image| image["kind"] == "url");
            if external && !lines.contains(&line) {
                lines.push(line);
            }
            map.values().for_each(|child| collect_external_backgrounds(child, line, lines));
        }
        _ => {}
    }
}
