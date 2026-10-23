use std::rc::Rc;

use gpui::{
    App, ElementId, ImageSource, InteractiveElement, IntoElement, ObjectFit, ParentElement,
    RenderOnce, SharedString, Styled, Window, div,
};

use crate::{
    buttons::SegmentedControl,
    forms::{Grid, OnValue, finder, grid, rows},
    primitives::Image,
    theme::{ActiveTheme, ControlSize},
};

/// Stickers across a picker's row, so it fits a narrow window.
const COLUMNS: usize = 4;

/// A sticker: its key, its name, and its square picture.
#[derive(Clone)]
pub struct Sticker {
    /// Tells it apart from the others.
    pub key: SharedString,
    /// What it is called; a search reads it.
    pub name: SharedString,
    /// The sticker itself.
    pub picture: ImageSource,
}

impl Sticker {
    /// A sticker keyed `key`, named `name`.
    pub fn new(
        key: impl Into<SharedString>,
        name: impl Into<SharedString>,
        picture: impl Into<ImageSource>,
    ) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            picture: picture.into(),
        }
    }
}

/// Stickers under their pack's name.
#[derive(Clone)]
pub struct StickerPack {
    /// The pack's name.
    pub name: SharedString,
    /// Its stickers.
    pub stickers: Vec<Sticker>,
}

impl StickerPack {
    /// A pack named `name` holding `stickers`.
    pub fn new(name: impl Into<SharedString>, stickers: impl IntoIterator<Item = Sticker>) -> Self {
        Self {
            name: name.into(),
            stickers: stickers.into_iter().collect(),
        }
    }
}

/// Stickers in a grid, a pack at a time, chosen above; a search spans every pack and keeps the stickers whose names hold it. Arrows move, and Enter or a press picks.
///
/// ```
/// use ely_messaging::{Sticker, StickerPack, StickerPicker};
///
/// let pack = StickerPack::new("Cats", [Sticker::new("wave", "Waving cat", "stickers/wave.png")]);
/// let picker = StickerPicker::new("stickers", [pack]).on_pick(|_key, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct StickerPicker {
    id: ElementId,
    packs: Vec<StickerPack>,
    on_pick: Option<OnValue>,
}

impl StickerPicker {
    /// A picker under `id` over `packs`; panics on none, or on a pack name found twice.
    pub fn new(id: impl Into<ElementId>, packs: impl IntoIterator<Item = StickerPack>) -> Self {
        let packs: Vec<StickerPack> = packs.into_iter().collect();
        assert!(!packs.is_empty(), "a sticker picker holds a pack");
        for (ix, pack) in packs.iter().enumerate() {
            let twice = packs[..ix].iter().any(|other| other.name == pack.name);
            assert!(!twice, "pack {} twice", pack.name);
        }
        let keys: Vec<&SharedString> = packs
            .iter()
            .flat_map(|pack| pack.stickers.iter().map(|sticker| &sticker.key))
            .collect();
        for (ix, key) in keys.iter().enumerate() {
            assert!(!keys[..ix].contains(key), "sticker {key} twice");
        }
        Self {
            id: id.into(),
            packs,
            on_pick: None,
        }
    }

    /// Gets the key of the sticker picked.
    pub fn on_pick(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_pick = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for StickerPicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let first = self.packs[0].name.clone();
        let chosen = window.use_keyed_state((self.id.clone(), "pack"), cx, |_, _| first.clone());
        if !self.packs.iter().any(|pack| pack.name == *chosen.read(cx)) {
            log::info!("sticker picker {}: pack gone, showing {first}", self.id);
            chosen.update(cx, |chosen, _| *chosen = first);
        }
        let pack = chosen.read(cx).clone();
        let finder = finder(&self.id, "Search stickers", window, cx);
        let query = finder.read(cx).query.clone();
        let packs = (self.packs.len() > 1).then(|| {
            let (id, choose) = (self.id.clone(), chosen.clone());
            self.packs
                .iter()
                .fold(
                    SegmentedControl::new((self.id.clone(), "packs"), pack.clone()),
                    |packs, each| packs.segment(each.name.clone(), each.name.clone(), None),
                )
                .size(ControlSize::Sm)
                .on_change(move |name, _, cx| {
                    log::info!("sticker picker {id}: pack {name}");
                    choose.update(cx, |chosen, cx| {
                        *chosen = name.clone();
                        cx.notify();
                    })
                })
        });
        let found: Rc<Vec<Sticker>> = Rc::new(
            self.packs
                .into_iter()
                .filter(|each| !query.is_empty() || each.name == pack)
                .flat_map(|each| each.stickers)
                .filter(|sticker| sticker.name.to_lowercase().contains(&query))
                .collect(),
        );
        let (painted, picked) = (found.clone(), found.clone());
        let (id, owner, on_pick) = (self.id.clone(), self.id.clone(), self.on_pick);
        let stickers = grid(
            Grid {
                id: self.id,
                names: Rc::new(found.iter().map(|sticker| sticker.name.clone()).collect()),
                rows: Rc::new(rows([(None, found.len())], COLUMNS)),
                selected: None,
                draw: Rc::new(move |ix, _| {
                    let sticker = &painted[ix];
                    div()
                        .size_full()
                        .p_1()
                        .child(
                            Image::new(
                                (owner.clone(), format!("sticker-{}", sticker.key)),
                                sticker.picture.clone(),
                            )
                            .fit(ObjectFit::Contain)
                            .size_full(),
                        )
                        .into_any_element()
                }),
                pick: Rc::new(move |ix, window, cx| {
                    let key = &picked[ix].key;
                    log::info!("sticker picker {id}: {key}");
                    if let Some(on_pick) = &on_pick {
                        on_pick(key, window, cx);
                    }
                }),
                none: "No stickers match",
                columns: COLUMNS,
                cell: cx.theme().messaging().sticker,
            },
            &finder,
            window,
            cx,
        );
        div()
            .debug_selector(|| "sticker-picker".into())
            .flex_none()
            .flex()
            .flex_col()
            .gap_2()
            .children(packs)
            .child(stickers)
    }
}
