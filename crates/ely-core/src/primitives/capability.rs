//! What many components can do, as traits generic code can name.

use gpui::{App, Window};

use crate::theme::ControlSize;

/// A component drawn at one of the theme's control sizes.
pub trait Sizable: Sized {
    /// Draws it at `size`.
    fn size(self, size: ControlSize) -> Self;
}

/// A component that can be turned off.
pub trait Disableable: Sized {
    /// Turns it off, or on again.
    fn disabled(self, disabled: bool) -> Self;
}

/// A component that can show itself chosen.
pub trait Selectable: Sized {
    /// Shows it chosen, or not.
    fn selected(self, selected: bool) -> Self;

    /// Whether it shows itself chosen.
    fn is_selected(&self) -> bool;
}

/// A component that tells its owner when its value changes.
pub trait Changeable: Sized {
    /// What the handler receives.
    type Value<'a>;

    /// Runs `handler` with each new value.
    fn on_change(self, handler: impl Fn(Self::Value<'_>, &mut Window, &mut App) + 'static) -> Self;
}

/// Implements capability traits by the component's own methods of the same name.
#[doc(hidden)]
#[macro_export]
macro_rules! capable {
    ($ty:ty: $($cap:ident $(($($value:tt)*))?),+ $(,)?) => {
        $($crate::capable!(@ $ty, $cap $(($($value)*))?);)+
    };
    (@ $ty:ty, size) => {
        impl $crate::primitives::Sizable for $ty {
            fn size(self, size: $crate::theme::ControlSize) -> Self {
                <$ty>::size(self, size)
            }
        }
    };
    (@ $ty:ty, disabled) => {
        impl $crate::primitives::Disableable for $ty {
            fn disabled(self, disabled: bool) -> Self {
                <$ty>::disabled(self, disabled)
            }
        }
    };
    (@ $ty:ty, change(& $value:ty)) => {
        impl $crate::primitives::Changeable for $ty {
            type Value<'a> = &'a $value;

            fn on_change(
                self,
                handler: impl Fn(&$value, &mut gpui::Window, &mut gpui::App) + 'static,
            ) -> Self {
                <$ty>::on_change(self, handler)
            }
        }
    };
    (@ $ty:ty, change($value:ty)) => {
        impl $crate::primitives::Changeable for $ty {
            type Value<'a> = $value;

            fn on_change(
                self,
                handler: impl Fn($value, &mut gpui::Window, &mut gpui::App) + 'static,
            ) -> Self {
                <$ty>::on_change(self, handler)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::{Changeable, Disableable, Selectable, Sizable};
    use crate::{
        buttons::Button,
        forms::{Slider, Switch},
        lists::ListItem,
        theme::ControlSize,
    };

    fn small_and_off<T: Sizable + Disableable>(control: T) -> T {
        control.size(ControlSize::Sm).disabled(true)
    }

    fn logged<T: Changeable>(control: T) -> T
    where
        for<'a> T::Value<'a>: Debug,
    {
        control.on_change(|value, _, _| log::info!("changed to {value:?}"))
    }

    fn chosen<T: Selectable>(item: T, selected: bool) -> bool {
        item.selected(selected).is_selected()
    }

    #[test]
    fn generic_code_drives_any_capable_component() {
        let _ = small_and_off(Button::new("go", "Go"));
        let _ = logged(Switch::new("wifi", true).disabled(false));
        let _ = logged(Slider::new("volume", 0.4));
        assert!(chosen(ListItem::new("row", "Row"), true));
        assert!(!chosen(ListItem::new("row", "Row"), false));
    }
}
