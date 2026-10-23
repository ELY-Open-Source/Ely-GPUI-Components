use ely_gpui_component::{
    forms::{
        CheckState, Checkbox, CheckboxCard, CheckboxGroup, Choice, RadioCard, RadioGroup, Switch,
    },
    primitives::IconName,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{code, keep, section, set, specimen},
};

fn values(list: &[&'static str]) -> Vec<SharedString> {
    list.iter()
        .map(|value| SharedString::from(*value))
        .collect()
}

pub fn checkboxes(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let channels = [
        Choice::new("email", "Email"),
        Choice::new("push", "Push"),
        Choice::new("sms", "Text message").disabled(),
    ];
    let open: Vec<SharedString> = channels
        .iter()
        .filter(|choice| !choice.disabled)
        .map(|choice| choice.value.clone())
        .collect();
    let on = keep("check-channels", || values(&["email"]), window, cx);
    let now = on.read(cx).clone();
    let parent = match now.len() {
        0 => CheckState::Off,
        n if n == open.len() => CheckState::On,
        _ => CheckState::Mixed,
    };
    let (all, some) = (on.clone(), on.clone());
    section(
        "Checkbox / CheckboxGroup",
        "On, off, or mixed when a parent's children disagree. The check pops in.",
        cx,
    )
    .child(probe(
        "checks",
        div()
            .flex()
            .flex_col()
            .gap_2()
            .w(px(280.0))
            .child(
                Checkbox::new("check-all", parent)
                    .label("All notifications")
                    .on_change(move |on, _, cx| {
                        set(&all, if on { open.clone() } else { Vec::new() }, cx)
                    }),
            )
            .child(
                div().pl_6().child(
                    CheckboxGroup::new("check-group", channels)
                        .selected(now)
                        .on_change(move |next, _, cx| set(&some, next.to_vec(), cx)),
                ),
            ),
    ))
    .child(code("Checkbox::new(id, CheckState::Mixed)", cx))
}

pub fn radios(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let plan = keep("radio-plan", || SharedString::from("pro"), window, cx);
    let density = keep(
        "radio-density",
        || SharedString::from("standard"),
        window,
        cx,
    );
    let (plan_now, density_now) = (plan.read(cx).clone(), density.read(cx).clone());
    section(
        "Radio / RadioGroup",
        "One Tab stop per group; the arrow keys move the choice.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(specimen(
                "plan",
                probe(
                    "radios",
                    RadioGroup::new(
                        "radio-plan",
                        [
                            Choice::new("free", "Free"),
                            Choice::new("pro", "Pro"),
                            Choice::new("team", "Team"),
                            Choice::new("enterprise", "Enterprise").disabled(),
                        ],
                    )
                    .selected(plan_now)
                    .on_change(move |value, _, cx| set(&plan, value.clone(), cx)),
                ),
                cx,
            ))
            .child(specimen(
                "density, in a row",
                RadioGroup::new(
                    "radio-density",
                    [
                        Choice::new("compact", "Compact"),
                        Choice::new("standard", "Standard"),
                        Choice::new("comfortable", "Comfortable"),
                    ],
                )
                .horizontal()
                .selected(density_now)
                .on_change(move |value, _, cx| set(&density, value.clone(), cx)),
                cx,
            )),
    )
}

pub fn cards(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let space = keep("card-space", || SharedString::from("personal"), window, cx);
    let backup = keep("card-backup", || true, window, cx);
    let (space_now, backup_now) = (space.read(cx).clone(), *backup.read(cx));
    let (personal, team) = (space.clone(), space);
    section(
        "CheckboxCard / RadioCard",
        "Larger targets that explain a choice. The chosen border turns to ink.",
        cx,
    )
    .child(probe(
        "cards",
        div()
            .flex()
            .gap_3()
            .w(px(640.0))
            .child(
                div().flex_1().child(
                    RadioCard::new("card-personal", space_now == "personal", "Personal")
                        .description("Just you. Notes stay on this Mac.")
                        .icon(IconName::User)
                        .on_select(move |_, cx| set(&personal, "personal".into(), cx)),
                ),
            )
            .child(
                div().flex_1().child(
                    RadioCard::new("card-team", space_now == "team", "Team")
                        .description("Shared with everyone you invite.")
                        .icon(IconName::Users)
                        .on_select(move |_, cx| set(&team, "team".into(), cx)),
                ),
            ),
    ))
    .child(
        div().w(px(640.0)).child(
            CheckboxCard::new("card-backup", backup_now, "Back up nightly")
                .description("Copies change sets to the cloud at 2 AM.")
                .icon(IconName::Cloud)
                .on_change(move |on, _, cx| set(&backup, on, cx)),
        ),
    )
}

pub fn switches(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let wifi = keep("switch-wifi", || true, window, cx);
    let sync = keep("switch-sync", || false, window, cx);
    let (wifi_now, sync_now) = (*wifi.read(cx), *sync.read(cx));
    section(
        "Switch / Toggle",
        "Takes effect at once. The thumb slides with a little overshoot.",
        cx,
    )
    .child(probe(
        "switches",
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                Switch::new("switch-wifi", wifi_now)
                    .label("Wi-Fi")
                    .on_change(move |on, _, cx| set(&wifi, on, cx)),
            )
            .child(
                Switch::new("switch-sync", sync_now)
                    .label("Sync over cellular")
                    .on_change(move |on, _, cx| set(&sync, on, cx)),
            )
            .child(
                Switch::new("switch-locked", true)
                    .label("Managed by your admin")
                    .disabled(true),
            ),
    ))
}
