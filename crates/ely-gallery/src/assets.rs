use std::borrow::Cow;

use ely_gpui_component::{Assets, primitives::IconName};
use gpui::{AssetSource, SharedString};

pub const CUSTOM_ICON: &str = "app-icons/mark.svg";
/// An arrow for the dial, as a host's own SVG.
pub const ARROW_ICON: &str = "app-icons/arrow-up.svg";

pub struct GalleryAssets;

impl AssetSource for GalleryAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        match path {
            CUSTOM_ICON => Ok(Some(Cow::Borrowed(include_bytes!(
                "../examples/assets/mark.svg"
            )))),
            ARROW_ICON => Ok(Some(Cow::Borrowed(IconName::ArrowUp.svg()))),
            _ => Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let mut assets = Assets.list(path)?;
        for own in [CUSTOM_ICON, ARROW_ICON] {
            if own.starts_with(path) {
                assets.push(own.into());
            }
        }
        Ok(assets)
    }
}
