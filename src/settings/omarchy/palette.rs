use anyhow::{Result, bail};
use gpui::{Hsla, Rgba, hsla};

use super::colors::{Colors, HUE_NAMES};
use crate::{
    devtools::{MARK_CONTRAST, TEXT_CONTRAST, audit},
    theme::{Mix, Mode, Palette, contrast},
};

const BLACK: Hsla = hsla(0.0, 0.0, 0.0, 1.0);
const WHITE: Hsla = hsla(0.0, 0.0, 1.0, 1.0);

/// How far each surface leans from the page toward the text.
const SURFACE: f32 = 0.04;
const OVERLAY: f32 = 0.06;
const SUNKEN: f32 = 0.08;
const HOVER: f32 = 0.08;
const ACTIVE: f32 = 0.14;
const SUBTLE: f32 = 0.16;

/// `color` toward `via`, then on to white or black, the one that reads better first, until it reads `need`:1 on every ground.
fn lift(color: Hsla, grounds: &[Hsla], via: Hsla, need: f32) -> Hsla {
    let worst = |color: Hsla| {
        grounds
            .iter()
            .map(|g| contrast(color, *g))
            .fold(f32::MAX, f32::min)
    };
    let ends = if worst(WHITE) > worst(BLACK) {
        [WHITE, BLACK]
    } else {
        [BLACK, WHITE]
    };
    let toward = |from: Hsla, to: Hsla, from_step: u32| {
        (from_step..=20).map(move |step| from.mix(&to, step as f32 / 20.0))
    };
    ends.into_iter()
        .flat_map(|end| toward(color, via, 0).chain(toward(via, end, 1)))
        .find(|candidate| worst(*candidate) >= need)
        .unwrap_or(ends[0])
}

/// The clearest wash that shows `color` over `page`, for a selection drawn over content.
pub(super) fn wash(color: Hsla, page: Hsla) -> Hsla {
    let (c, p) = (Rgba::from(color), Rgba::from(page));
    let reach = |c: f32, p: f32| match c - p {
        d if d > 0.0 => d / (1.0 - p),
        d if d < 0.0 => -d / p,
        _ => 0.0,
    };
    let alpha = reach(c.r, p.r).max(reach(c.g, p.g)).max(reach(c.b, p.b));
    if alpha == 0.0 {
        return color.alpha(0.0);
    }
    let channel = |c: f32, p: f32| (p + (c - p) / alpha).clamp(0.0, 1.0);
    Hsla::from(Rgba {
        r: channel(c.r, p.r),
        g: channel(c.g, p.g),
        b: channel(c.b, p.b),
        a: alpha,
    })
}

/// The palette, and the hues Ely's colors stand in for.
pub(super) fn build(c: &Colors) -> Result<(Palette, Vec<&'static str>)> {
    let mut p = match c.mode {
        Mode::Dark => Palette::dark(false),
        Mode::Light => Palette::light(false),
    };
    let bg = c.bg;
    let tint = |t: f32| bg.mix(&c.fg, t);
    let grounds = [
        bg,
        tint(SURFACE),
        tint(OVERLAY),
        tint(SUNKEN),
        tint(HOVER),
        tint(ACTIVE),
    ];
    let fg = lift(c.fg, &grounds, c.fg, TEXT_CONTRAST);
    let lifted = |color: Hsla, grounds: &[Hsla], need: f32| lift(color, grounds, fg, need);
    let text = |color: Hsla| lifted(color, &grounds, TEXT_CONTRAST);

    let mut missing = Vec::new();
    let mut hue = |i: usize, ely: Hsla| {
        c.hues[i].unwrap_or_else(|| {
            missing.push(HUE_NAMES[i]);
            ely
        })
    };
    let (red, green, yellow, blue) = (
        hue(0, p.danger),
        hue(1, p.success),
        hue(2, p.warning),
        hue(3, p.info),
    );
    let (magenta, cyan) = (hue(4, p.chart[4]), hue(5, p.chart[1]));
    let orange = c.orange.unwrap_or(yellow);
    let accent = c.accent.or(c.hues[3]).unwrap_or_else(|| {
        missing.push("accent");
        p.accent
    });
    let bright = |i: usize, hue: Hsla| c.bright[i].unwrap_or(hue);

    p.bg = bg;
    (p.surface, p.overlay, p.sunken, p.hover, p.active) =
        (grounds[1], grounds[2], grounds[3], grounds[4], grounds[5]);
    p.border = tint(0.16);
    p.border_strong = tint(0.32);
    p.fg = fg;
    p.fg_muted = text(fg.mix(&bg, 0.25));
    p.fg_subtle = text(fg.mix(&bg, 0.45));
    p.fg_disabled = fg.mix(&bg, 0.65);
    p.accent = accent;
    p.on_accent = if contrast(WHITE, accent) > contrast(BLACK, accent) {
        WHITE
    } else {
        BLACK
    };
    let away = if p.on_accent == WHITE { BLACK } else { WHITE };
    p.accent_hover = [accent.mix(&fg, 0.12), accent.mix(&away, 0.12)]
        .into_iter()
        .find(|hover| contrast(p.on_accent, *hover) >= TEXT_CONTRAST)
        .unwrap_or(accent);
    p.focus = lifted(accent, &grounds[..2], MARK_CONTRAST);
    p.link = text(accent);
    p.selection = wash(c.selection, bg);
    p.success_subtle = bg.mix(&green, SUBTLE);
    p.warning_subtle = bg.mix(&yellow, SUBTLE);
    p.danger_subtle = bg.mix(&red, SUBTLE);
    p.info_subtle = bg.mix(&blue, SUBTLE);
    p.success = lifted(green, &[bg, p.success_subtle], TEXT_CONTRAST);
    p.warning = lifted(yellow, &[bg, p.warning_subtle], TEXT_CONTRAST);
    p.danger = lifted(red, &[bg, p.danger_subtle], TEXT_CONTRAST);
    p.info = lifted(blue, &[bg, p.info_subtle], TEXT_CONTRAST);
    p.tooltip_bg = fg;
    p.tooltip_fg = bg;
    p.chart = [
        blue,
        cyan,
        yellow,
        red,
        magenta,
        green,
        orange,
        bright(5, cyan),
    ]
    .map(|hue| lifted(hue, &[bg], MARK_CONTRAST));
    p.ansi = [
        bg,
        red,
        green,
        yellow,
        blue,
        magenta,
        cyan,
        c.fg,
        c.muted,
        bright(0, red),
        bright(1, green),
        bright(2, yellow),
        bright(3, blue),
        bright(4, magenta),
        bright(5, cyan),
        c.bright_fg,
    ];
    let s = &mut p.syntax;
    s.keyword = text(magenta);
    s.string = text(green);
    s.number = text(orange);
    s.comment = p.fg_subtle;
    s.function = text(blue);
    s.type_name = text(yellow);
    s.constant = text(orange);
    s.property = text(cyan);
    s.tag = text(red);
    s.attribute = text(yellow);
    s.operator = text(cyan);
    s.punctuation = p.fg_muted;
    s.variable = fg;
    if let Some(check) = audit(&p).into_iter().find(|check| !check.passes()) {
        bail!(
            "{} on {} reads {:.2}:1, needs {}:1",
            check.front,
            check.back,
            check.ratio,
            check.need
        );
    }
    Ok((p, missing))
}
