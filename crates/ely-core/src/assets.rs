use std::borrow::Cow;

use anyhow::Context as _;
use gpui::{App, AssetSource, Result, SharedString};

/// Ely's fonts. Pass to `Application::with_assets`, alone or through `Assets::before`.
pub struct Assets;

/// IBM Plex Sans, where gpui's svg renderer loads it.
const PLEX: &[(&str, &[u8])] = &[
    (
        "fonts/ibm-plex-sans/IBMPlexSans-LICENSE.txt",
        include_bytes!("../assets/plex/IBMPlexSans-LICENSE.txt"),
    ),
    (
        "fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf",
        include_bytes!("../assets/plex/IBMPlexSans-Regular.ttf"),
    ),
];

/// Inter and JetBrains Mono, the theme's families.
#[cfg(feature = "fonts")]
const FONTS: &[(&str, &[u8])] = &[
    (
        "fonts/Inter-Italic.ttf",
        include_bytes!("../assets/fonts/Inter-Italic.ttf"),
    ),
    (
        "fonts/Inter-LICENSE.txt",
        include_bytes!("../assets/fonts/Inter-LICENSE.txt"),
    ),
    (
        "fonts/Inter-Medium.ttf",
        include_bytes!("../assets/fonts/Inter-Medium.ttf"),
    ),
    (
        "fonts/Inter-Regular.ttf",
        include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    ),
    (
        "fonts/Inter-SemiBold.ttf",
        include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
    ),
    (
        "fonts/JetBrainsMono-LICENSE.txt",
        include_bytes!("../assets/fonts/JetBrainsMono-LICENSE.txt"),
    ),
    (
        "fonts/JetBrainsMono-Medium.ttf",
        include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf"),
    ),
    (
        "fonts/JetBrainsMono-Regular.ttf",
        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    ),
];

impl Assets {
    fn files() -> impl Iterator<Item = &'static (&'static str, &'static [u8])> {
        let files = PLEX.iter();
        #[cfg(feature = "fonts")]
        let files = files.chain(FONTS);
        files
    }

    fn get(path: &str) -> Option<Cow<'static, [u8]>> {
        Self::files()
            .find(|(name, _)| *name == path)
            .map(|(_, data)| Cow::Borrowed(*data))
    }

    fn iter() -> impl Iterator<Item = Cow<'static, str>> {
        Self::files().map(|(name, _)| Cow::Borrowed(*name))
    }
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(Self::get(path))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter(|name| name.starts_with(path))
            .map(SharedString::from)
            .collect())
    }
}

pub(crate) fn load_fonts(cx: &App) -> Result<()> {
    let fonts = Assets::iter()
        .filter(|name| name.ends_with(".ttf"))
        .map(|name| {
            Assets::get(&name).with_context(|| format!("font {name} vanished from the bundle"))
        })
        .collect::<Result<Vec<_>>>()?;
    log::info!("assets: registering {} fonts", fonts.len());
    cx.text_system().add_fonts(fonts)
}

/// Ely's files first, then another source's: one source for an app that runs both.
pub struct Before<T>(T);

impl Assets {
    /// Serves Ely's files, and every other path from `other`, such as GPUI Kit's `Assets`.
    pub fn before<T: AssetSource>(other: T) -> Before<T> {
        Before(other)
    }
}

impl<T: AssetSource> AssetSource for Before<T> {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match Assets.load(path)? {
            Some(data) => Ok(Some(data)),
            None => self.0.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut names = Assets.list(path)?;
        names.extend(self.0.list(path)?);
        names.sort();
        names.dedup();
        Ok(names)
    }
}

/// Ely's files the source cannot load, each with the reason.
pub(crate) fn missing(source: &dyn AssetSource) -> Vec<String> {
    Assets::iter()
        .filter_map(|path| match source.load(&path) {
            Ok(Some(_)) => None,
            Ok(None) => Some(format!("{path}: not found")),
            Err(error) => Some(format!("{path}: {error}")),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use gpui::{AssetSource, Result, SharedString};

    use super::{Assets, missing};

    const PLEX: &str = "fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf";

    /// Icons alone, and an error for any other path, as GPUI Kit's source answers.
    struct IconsOnly;

    impl AssetSource for IconsOnly {
        fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
            match path {
                "icons/check.svg" | "icons/kit-only.svg" => Ok(Some(Cow::Borrowed(b"<svg/>"))),
                _ => Err(anyhow::anyhow!("could not find asset at path {path}")),
            }
        }

        fn list(&self, _: &str) -> Result<Vec<SharedString>> {
            Ok(vec!["icons/check.svg".into(), "icons/kit-only.svg".into()])
        }
    }

    #[test]
    fn ely_s_own_source_holds_every_file() {
        assert_eq!(missing(&Assets), Vec::<String>::new());
    }

    #[test]
    fn an_icons_only_source_lacks_ely_s_fonts() {
        let missing = missing(&IconsOnly);
        assert!(
            missing.iter().any(|line| line.starts_with(PLEX)),
            "{missing:?}"
        );
    }

    #[test]
    fn ely_s_files_come_first_then_the_other_s() {
        let both = Assets::before(IconsOnly);
        assert_eq!(missing(&both), Vec::<String>::new());
        let plex = both.load(PLEX).unwrap().unwrap();
        assert_eq!(plex, Assets.load(PLEX).unwrap().unwrap());
        assert_eq!(
            &*both.load("icons/kit-only.svg").unwrap().unwrap(),
            b"<svg/>"
        );
        assert!(both.load("icons/neither.svg").is_err());
        let icons = both.list("icons/").unwrap();
        let checks = icons
            .iter()
            .filter(|name| *name == "icons/check.svg")
            .count();
        assert_eq!(checks, 1);
        assert!(icons.iter().any(|name| name == "icons/kit-only.svg"));
    }
}
