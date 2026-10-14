use crate::text::tr;
use crate::ir::{Border, Enter, GradientStop, HAlign, ImageSource, Length, Paint, Shadow, Transition, VAlign};
use crate::theme::Theme;

pub struct Invalid {
    pub message: String,
    pub hint: Option<String>,
}

pub type Parsed<T> = Result<T, Invalid>;

fn invalid<T>(message: impl Into<String>, hint: impl Into<String>) -> Parsed<T> {
    Err(Invalid { message: message.into(), hint: Some(hint.into()) })
}

pub const SPACE_SCALE: &[(&str, f32)] =
    &[("none", 0.0), ("xs", 8.0), ("sm", 16.0), ("md", 32.0), ("lg", 48.0), ("xl", 72.0), ("2xl", 112.0), ("3xl", 160.0)];

pub const SIZE_SCALE: &[(&str, f32)] = &[
    ("xs", 20.0),
    ("sm", 26.0),
    ("md", 32.0),
    ("lg", 40.0),
    ("xl", 52.0),
    ("2xl", 64.0),
    ("3xl", 88.0),
    ("4xl", 116.0),
    ("5xl", 150.0),
    ("6xl", 200.0),
    ("7xl", 260.0),
];

pub const RADIUS_SCALE: &[(&str, f32)] = &[("none", 0.0), ("sm", 8.0), ("md", 16.0), ("lg", 28.0), ("xl", 44.0), ("full", 9999.0)];

const WEIGHTS: &[(&str, u16)] =
    &[("thin", 100), ("light", 300), ("regular", 400), ("medium", 500), ("semibold", 600), ("bold", 700), ("extrabold", 800), ("black", 900)];

fn names(scale: &[(&str, f32)]) -> String {
    scale.iter().map(|(name, _)| *name).collect::<Vec<_>>().join(", ")
}

fn number(value: &str) -> Option<f32> {
    value.strip_suffix("px").unwrap_or(value).parse::<f32>().ok().filter(|n| n.is_finite())
}

fn scaled(value: &str, scale: &[(&str, f32)], min: f32, max: f32, what: &str) -> Parsed<f32> {
    if let Some((_, px)) = scale.iter().find(|(name, _)| *name == value) {
        return Ok(*px);
    }
    match number(value) {
        Some(n) if (min..=max).contains(&n) => Ok(n),
        Some(_) => invalid(tr(format!("{what} `{value}` fora do intervalo ({min} a {max})"), format!("{what} `{value}` out of range ({min} to {max})")), tr(format!("Use um valor entre {min} e {max} ou um destes: {}", names(scale)), format!("Use a value between {min} and {max} or one of: {}", names(scale)))),
        None => invalid(tr(format!("{what} inválido: `{value}`"), format!("invalid {what}: `{value}`")), tr(format!("Use um destes: {} ou um número em pixels", names(scale)), format!("Use one of: {} or a number in pixels", names(scale)))),
    }
}

pub fn space(value: &str) -> Parsed<f32> {
    scaled(value, SPACE_SCALE, 0.0, 600.0, &tr("espaçamento", "spacing"))
}

pub fn size(value: &str) -> Parsed<f32> {
    scaled(value, SIZE_SCALE, 8.0, 400.0, &tr("tamanho", "size"))
}

pub fn radius(value: &str) -> Parsed<f32> {
    scaled(value, RADIUS_SCALE, 0.0, 9999.0, &tr("arredondamento", "radius"))
}

pub fn pad(value: &str) -> Parsed<[f32; 2]> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [all] => space(all).map(|n| [n, n]),
        [y, x] => Ok([space(y)?, space(x)?]),
        _ => invalid(tr(format!("espaçamento interno inválido: `{value}`"), format!("invalid padding: `{value}`")), tr("Use um valor (pad=md) ou dois, vertical e horizontal (pad=\"sm lg\")", "Use one value (pad=md) or two, vertical and horizontal (pad=\"sm lg\")")),
    }
}

pub fn color(value: &str, theme: &Theme) -> Parsed<String> {
    let (base, alpha) = match value.split_once('/') {
        Some((base, alpha)) => match alpha.parse::<u8>() {
            Ok(alpha) if alpha <= 100 => (base, Some(alpha)),
            _ => return invalid(tr(format!("opacidade inválida em `{value}`"), format!("invalid opacity in `{value}`")), tr("Use um número de 0 a 100 depois da barra, como coral/40", "Use a number from 0 to 100 after the slash, such as coral/40")),
        },
        None => (value, None),
    };
    let hex = if base.starts_with('#') {
        normalize_hex(base).ok_or_else(|| Invalid {
            message: tr(format!("cor hexadecimal inválida: `{base}`"), format!("invalid hex color: `{base}`")),
            hint: Some(tr("Use #rgb, #rrggbb ou #rrggbbaa", "Use #rgb, #rrggbb or #rrggbbaa")),
        })?
    } else if let Some(hex) = theme.colors.get(base) {
        hex.clone()
    } else {
        let known = theme.colors.keys().map(String::as_str).collect::<Vec<_>>().join(", ");
        let suggestion = crate::text::suggest(base, theme.colors.keys().map(String::as_str));
        return Err(Invalid {
            message: tr(format!("cor desconhecida: `{base}`"), format!("unknown color: `{base}`")),
            hint: Some(match suggestion {
                Some(name) => tr(format!("Você quis dizer `{name}`? Cores disponíveis: {known}"), format!("Did you mean `{name}`? Available colors: {known}")),
                None => tr(format!("Cores disponíveis: {known}, ou use #rrggbb"), format!("Available colors: {known}, or use #rrggbb")),
            }),
        });
    };
    Ok(match alpha {
        Some(alpha) => with_alpha(&hex, f32::from(alpha) / 100.0),
        None => hex,
    })
}

fn normalize_hex(value: &str) -> Option<String> {
    let digits = value.strip_prefix('#')?;
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let expanded = match digits.len() {
        3 | 4 => digits.chars().flat_map(|c| [c, c]).collect(),
        6 | 8 => digits.to_string(),
        _ => return None,
    };
    Some(format!("#{}", expanded.to_ascii_lowercase()))
}

fn with_alpha(hex: &str, factor: f32) -> String {
    let digits = &hex[1..];
    let base_alpha = if digits.len() == 8 { u8::from_str_radix(&digits[6..8], 16).unwrap_or(255) } else { 255 };
    let alpha = (f32::from(base_alpha) * factor).round() as u8;
    format!("#{}{:02x}", &digits[..6], alpha)
}

pub fn background(value: &str, theme: &Theme) -> Parsed<Paint> {
    match value.split_once("..") {
        Some((from, to)) => Ok(Paint::Linear {
            angle: 135.0,
            stops: vec![GradientStop { color: color(from, theme)?, at: 0.0 }, GradientStop { color: color(to, theme)?, at: 1.0 }],
        }),
        None => color(value, theme).map(|color| Paint::Solid { color }),
    }
}

pub fn font(value: &str, theme: &Theme) -> Parsed<String> {
    match theme.fonts.get(value) {
        Some(spec) => Ok(spec.family.clone()),
        None => {
            let known = theme.fonts.keys().map(String::as_str).collect::<Vec<_>>().join(", ");
            invalid(tr(format!("fonte desconhecida: `{value}`"), format!("unknown font: `{value}`")), tr(format!("Fontes do tema: {known}. Declare outras com @font nome \"Família\""), format!("Theme fonts: {known}. Declare others with @font name \"Family\"")))
        }
    }
}

pub fn weight(value: &str) -> Parsed<u16> {
    if let Some((_, weight)) = WEIGHTS.iter().find(|(name, _)| *name == value) {
        return Ok(*weight);
    }
    match value.parse::<u16>() {
        Ok(weight) if (100..=900).contains(&weight) && weight % 100 == 0 => Ok(weight),
        _ => invalid(
            tr(format!("peso de fonte inválido: `{value}`"), format!("invalid font weight: `{value}`")),
            tr(format!("Use {} ou 100 a 900", WEIGHTS.iter().map(|(name, _)| *name).collect::<Vec<_>>().join(", ")), format!("Use {} or 100 to 900", WEIGHTS.iter().map(|(name, _)| *name).collect::<Vec<_>>().join(", "))),
        ),
    }
}

pub fn flag(value: Option<&str>) -> Parsed<bool> {
    match value {
        None | Some("true") | Some("yes") | Some("sim") => Ok(true),
        Some("false") | Some("no") | Some("não") => Ok(false),
        Some(other) => invalid(tr(format!("valor booleano inválido: `{other}`"), format!("invalid boolean value: `{other}`")), tr("Use apenas o nome do atributo para ativar, ou =false para desativar", "Use just the attribute name to enable it, or =false to disable it")),
    }
}

pub fn halign(value: &str) -> Parsed<HAlign> {
    match value {
        "left" => Ok(HAlign::Left),
        "center" => Ok(HAlign::Center),
        "right" => Ok(HAlign::Right),
        _ => invalid(tr(format!("alinhamento horizontal inválido: `{value}`"), format!("invalid horizontal alignment: `{value}`")), tr("Use left, center ou right", "Use left, center or right")),
    }
}

pub fn valign(value: &str) -> Parsed<VAlign> {
    match value {
        "top" => Ok(VAlign::Top),
        "center" | "middle" => Ok(VAlign::Center),
        "bottom" => Ok(VAlign::Bottom),
        _ => invalid(tr(format!("alinhamento vertical inválido: `{value}`"), format!("invalid vertical alignment: `{value}`")), tr("Use top, center ou bottom", "Use top, center or bottom")),
    }
}

pub fn shadow(value: Option<&str>) -> Parsed<Shadow> {
    match value.unwrap_or("md") {
        "none" => Ok(Shadow::None),
        "sm" => Ok(Shadow::Sm),
        "md" => Ok(Shadow::Md),
        "lg" => Ok(Shadow::Lg),
        other => invalid(tr(format!("sombra inválida: `{other}`"), format!("invalid shadow: `{other}`")), tr("Use sm, md, lg ou none", "Use sm, md, lg or none")),
    }
}

pub fn border(value: Option<&str>, theme: &Theme) -> Parsed<Border> {
    let mut width = 3.0;
    let mut border_color = theme.colors.get("line").cloned().unwrap_or_else(|| "#00000033".into());
    for part in value.unwrap_or("").split_whitespace() {
        match number(part) {
            Some(n) if (0.0..=40.0).contains(&n) => width = n,
            Some(_) => return invalid(tr(format!("espessura de borda fora do intervalo: `{part}`"), format!("border width out of range: `{part}`")), tr("Use um número entre 0 e 40", "Use a number between 0 and 40")),
            None => border_color = color(part, theme)?,
        }
    }
    Ok(Border { width, color: border_color })
}

pub fn length(value: &str, allow_auto: bool) -> Parsed<Length> {
    if allow_auto && value == "auto" {
        return Ok(Length::Auto);
    }
    if value == "full" {
        return Ok(Length::Pct { value: 100.0 });
    }
    if let Some(percent) = value.strip_suffix('%') {
        return match percent.parse::<f32>() {
            Ok(n) if (0.0..=100.0).contains(&n) => Ok(Length::Pct { value: n }),
            _ => invalid(tr(format!("porcentagem inválida: `{value}`"), format!("invalid percentage: `{value}`")), tr("Use um valor de 0% a 100%", "Use a value from 0% to 100%")),
        };
    }
    match number(value) {
        Some(n) if (-4000.0..=4000.0).contains(&n) => Ok(Length::Px { value: n }),
        _ => invalid(tr(format!("medida inválida: `{value}`"), format!("invalid size: `{value}`")), tr("Use pixels (480), porcentagem (50%), full ou auto", "Use pixels (480), a percentage (50%), full or auto")),
    }
}

pub fn opacity(value: &str) -> Parsed<f32> {
    let parsed = match value.strip_suffix('%') {
        Some(percent) => percent.parse::<f32>().ok().map(|n| n / 100.0),
        None => value.parse::<f32>().ok(),
    };
    match parsed {
        Some(n) if (0.0..=1.0).contains(&n) => Ok(n),
        _ => invalid(tr(format!("opacidade inválida: `{value}`"), format!("invalid opacity: `{value}`")), tr("Use um número de 0 a 1 ou uma porcentagem, como 60%", "Use a number from 0 to 1 or a percentage, such as 60%")),
    }
}

pub fn leading(value: &str) -> Parsed<f32> {
    match value {
        "tight" => Ok(1.05),
        "normal" => Ok(1.35),
        "loose" => Ok(1.7),
        _ => match value.parse::<f32>() {
            Ok(n) if (0.6..=3.0).contains(&n) => Ok(n),
            _ => invalid(tr(format!("entrelinha inválida: `{value}`"), format!("invalid line height: `{value}`")), tr("Use tight, normal, loose ou um número entre 0.6 e 3", "Use tight, normal, loose or a number between 0.6 and 3")),
        },
    }
}

pub fn tracking(value: &str) -> Parsed<f32> {
    match value {
        "tight" => Ok(-0.02),
        "normal" => Ok(0.0),
        "wide" => Ok(0.08),
        _ => match value.parse::<f32>() {
            Ok(n) if (-0.2..=1.0).contains(&n) => Ok(n),
            _ => invalid(tr(format!("espaçamento entre letras inválido: `{value}`"), format!("invalid letter spacing: `{value}`")), tr("Use tight, normal, wide ou um número em em, como 0.1", "Use tight, normal, wide or a number in em, such as 0.1")),
        },
    }
}

pub fn is_asset_reference(value: &str) -> bool {
    value.strip_prefix("asset:").is_some_and(|id| {
        id.len() == 36
            && id.char_indices().all(|(index, c)| if matches!(index, 8 | 13 | 18 | 23) { c == '-' } else { c.is_ascii_hexdigit() })
    })
}

pub fn url(value: &str, allow_data: bool) -> Parsed<String> {
    if allow_data && is_asset_reference(value) {
        return Ok(value.to_ascii_lowercase());
    }
    let lower = value.to_ascii_lowercase();
    let scheme_ok = lower.starts_with("https://")
        || lower.starts_with("http://")
        || (allow_data && ["data:image/png;", "data:image/jpeg;", "data:image/gif;", "data:image/webp;"].iter().any(|p| lower.starts_with(p)));
    let chars_ok = !value.chars().any(|c| c.is_whitespace() || c.is_control() || matches!(c, '"' | '\'' | '<' | '>' | '\\' | '(' | ')'));
    if scheme_ok && chars_ok {
        Ok(value.to_string())
    } else {
        invalid(
            tr(format!("endereço não permitido: `{value}`"), format!("address not allowed: `{value}`")),
            if allow_data { tr("Use uma imagem guardada (asset:...) ou um endereço que comece com https://", "Use a stored image (asset:...) or an address that starts with https://") } else { tr("Use um endereço que comece com https://", "Use an address that starts with https://") },
        )
    }
}

pub fn image_source(value: &str) -> Parsed<ImageSource> {
    let accepted = url(value, true)?;
    Ok(match accepted.strip_prefix("asset:") {
        Some(id) if is_asset_reference(&accepted) => ImageSource::Asset { id: id.to_string() },
        _ => ImageSource::Url { url: accepted },
    })
}

pub fn audio_source(value: &str) -> Parsed<ImageSource> {
    if is_asset_reference(value) {
        let lower = value.to_ascii_lowercase();
        return Ok(ImageSource::Asset { id: lower.trim_start_matches("asset:").to_string() });
    }
    url(value, false).map(|url| ImageSource::Url { url })
}

pub fn link(value: &str) -> Parsed<String> {
    let lower = value.to_ascii_lowercase();
    if lower.starts_with("mailto:") || (value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')) {
        return Ok(value.to_string());
    }
    url(value, false)
}

pub fn enter(value: &str) -> Parsed<Enter> {
    match value {
        "fade" => Ok(Enter::Fade),
        "up" => Ok(Enter::Up),
        "down" => Ok(Enter::Down),
        "left" => Ok(Enter::Left),
        "right" => Ok(Enter::Right),
        "zoom" => Ok(Enter::Zoom),
        "none" => Ok(Enter::None),
        other => invalid(tr(format!("entrada inválida: `{other}`"), format!("invalid entrance: `{other}`")), tr("Use fade, up, down, left, right, zoom ou none", "Use fade, up, down, left, right, zoom or none")),
    }
}

pub fn transition(value: &str) -> Parsed<Transition> {
    match value {
        "fade" => Ok(Transition::Fade),
        "slide" => Ok(Transition::Slide),
        "zoom" => Ok(Transition::Zoom),
        "none" => Ok(Transition::None),
        other => invalid(tr(format!("transição inválida: `{other}`"), format!("invalid transition: `{other}`")), tr("Use fade, slide, zoom ou none", "Use fade, slide, zoom or none")),
    }
}
