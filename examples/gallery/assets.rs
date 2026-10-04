use std::borrow::Cow;

use ely_gpui_component::Assets;
use gpui::{AssetSource, SharedString};

pub const CUSTOM_ICON: &str = "app-icons/mark.svg";

pub struct GalleryAssets;

impl AssetSource for GalleryAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        if path == CUSTOM_ICON {
            return Ok(Some(Cow::Borrowed(include_bytes!("../assets/mark.svg"))));
        }
        Assets.load(path)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let mut assets = Assets.list(path)?;
        if CUSTOM_ICON.starts_with(path) {
            assets.push(CUSTOM_ICON.into());
        }
        Ok(assets)
    }
}
