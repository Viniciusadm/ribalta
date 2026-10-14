use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::ir::{Block, BlockKind, IconElement, Page};

const BUNDLED: &str = include_str!("../icons/icons.json");

type IconSet = BTreeMap<String, Vec<IconElement>>;
type RawElement = (String, BTreeMap<String, String>);

fn library() -> &'static IconSet {
    static LIBRARY: OnceLock<IconSet> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let raw: BTreeMap<String, Vec<RawElement>> = serde_json::from_str(BUNDLED).expect("the bundled icon set is valid JSON");
        raw.into_iter().map(|(name, elements)| (name, elements.into_iter().map(|(tag, attrs)| IconElement { tag, attrs }).collect())).collect()
    })
}

pub fn is_icon(name: &str) -> bool {
    library().contains_key(name)
}

pub fn names() -> impl Iterator<Item = &'static str> {
    library().keys().map(String::as_str)
}

pub fn used_by(pages: &[Page]) -> IconSet {
    let mut used = IconSet::new();
    for page in pages {
        for block in page.children.iter().chain(&page.header).chain(&page.footer).chain(&page.overlay) {
            collect(block, &mut used);
        }
    }
    used
}

fn collect(block: &Block, used: &mut IconSet) {
    match &block.kind {
        BlockKind::Icon { name } => {
            if let Some(elements) = library().get(name) {
                used.entry(name.clone()).or_insert_with(|| elements.clone());
            }
        }
        BlockKind::Quote { children } | BlockKind::Layout { children, .. } | BlockKind::Each { children, .. } => children.iter().for_each(|child| collect(child, used)),
        BlockKind::List { items, .. } => items.iter().flat_map(|item| &item.children).for_each(|child| collect(child, used)),
        _ => {}
    }
}
