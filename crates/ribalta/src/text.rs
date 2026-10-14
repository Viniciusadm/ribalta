#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub line: usize,
    pub from: usize,
    pub to: usize,
}

impl Span {
    pub fn new(line: usize, from: usize, to: usize) -> Self {
        Self { line, from, to }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub value: String,
    pub span: Span,
}

impl Text {
    pub fn new(value: impl Into<String>, line: usize, from: usize) -> Self {
        let value = value.into();
        let to = from + value.len();
        Self { value, span: Span::new(line, from, to) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line<'a> {
    pub number: usize,
    pub offset: usize,
    pub text: &'a str,
}

impl<'a> Line<'a> {
    pub fn trimmed(&self) -> &'a str {
        self.text.trim()
    }

    pub fn indent(&self) -> usize {
        self.text.len() - self.text.trim_start().len()
    }

    pub fn is_blank(&self) -> bool {
        self.text.trim().is_empty()
    }

    pub fn slice(&self, from: usize) -> Line<'a> {
        Line { number: self.number, offset: self.offset + from, text: &self.text[from..] }
    }

    pub fn text_at(&self, from: usize, to: usize) -> Text {
        Text::new(&self.text[from..to], self.number, self.offset + from)
    }

    pub fn span(&self, from: usize, to: usize) -> Span {
        Span::new(self.number, self.offset + from, self.offset + to)
    }

    pub fn full_span(&self) -> Span {
        let start = self.indent();
        let end = self.text.trim_end().len().max(start);
        self.span(start, end)
    }
}

pub fn suggest<'a>(input: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    candidates
        .into_iter()
        .map(|candidate| (levenshtein(input, candidate), candidate))
        .filter(|(distance, candidate)| *distance <= 2.max(candidate.len() / 4) && *distance < candidate.len())
        .min_by_key(|(distance, candidate)| (*distance, *candidate))
        .map(|(_, candidate)| candidate)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            current.push((previous[j] + cost).min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b.len()]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    Pt,
    En,
}

impl Locale {
    pub fn from_code(code: &str) -> Self {
        if code.to_ascii_lowercase().starts_with("en") { Self::En } else { Self::Pt }
    }
}

thread_local! {
    static LOCALE: std::cell::Cell<Locale> = const { std::cell::Cell::new(Locale::Pt) };
}

pub fn with_locale<T>(locale: Locale, run: impl FnOnce() -> T) -> T {
    let previous = LOCALE.with(|cell| cell.replace(locale));
    let result = run();
    LOCALE.with(|cell| cell.set(previous));
    result
}

pub fn tr(pt: impl Into<String>, en: impl Into<String>) -> String {
    match LOCALE.with(std::cell::Cell::get) {
        Locale::Pt => pt.into(),
        Locale::En => en.into(),
    }
}
