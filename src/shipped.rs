//! Components Ely ships say nothing in fixed English.

/// Sources of components the Ely apps draw.
const SHIPPED: &[(&str, &str)] = &[
    (
        "editor/find.rs",
        include_str!("../crates/ely-editor/src/find.rs"),
    ),
    (
        "editor/hints.rs",
        include_str!("../crates/ely-editor/src/hints.rs"),
    ),
    (
        "forms/input.rs",
        include_str!("../crates/ely-core/src/forms/input.rs"),
    ),
    (
        "forms/path.rs",
        include_str!("../crates/ely-core/src/forms/path.rs"),
    ),
    (
        "forms/catalogs.rs",
        include_str!("../crates/ely-core/src/forms/catalogs.rs"),
    ),
    (
        "forms/select.rs",
        include_str!("../crates/ely-core/src/forms/select.rs"),
    ),
    (
        "forms/date/calendar.rs",
        include_str!("../crates/ely-core/src/forms/date/calendar.rs"),
    ),
    (
        "forms/date/picker.rs",
        include_str!("../crates/ely-core/src/forms/date/picker.rs"),
    ),
    (
        "mail/compose.rs",
        include_str!("../crates/ely-mail/src/compose.rs"),
    ),
    (
        "mail/signature.rs",
        include_str!("../crates/ely-mail/src/signature.rs"),
    ),
    (
        "chat/attach.rs",
        include_str!("../crates/ely-chat/src/attach.rs"),
    ),
    (
        "tables/builders.rs",
        include_str!("../crates/ely-core/src/tables/builders.rs"),
    ),
    (
        "tables/rules.rs",
        include_str!("../crates/ely-core/src/tables/rules.rs"),
    ),
    (
        "navigation/editor_tabs.rs",
        include_str!("../crates/ely-core/src/navigation/editor_tabs.rs"),
    ),
    (
        "settings/appearance.rs",
        include_str!("../crates/ely-settings/src/appearance.rs"),
    ),
];

#[test]
fn shipped_components_show_no_fixed_english() {
    let shown = [
        ".tooltip(\"",
        ".child(\"",
        ".placeholder(\"",
        "SharedString::from(\"",
        "=> \"",
    ];
    // After an id: words, not a lowercase id.
    let labels = ["), \"", "\", \"", "(\""];
    let mut found = Vec::new();
    for (path, source) in SHIPPED {
        let code = source.split("#[cfg(test)]").next().unwrap_or(source);
        for (no, line) in code.lines().enumerate() {
            let checks = ["assert", "panic!", "log::", ".expect(", "debug_selector"];
            let comment = line.trim_start().starts_with("//");
            if comment || checks.iter().any(|check| line.contains(check)) {
                continue;
            }
            let literals = |calls: &[&str]| -> Vec<&str> {
                (calls.iter())
                    .flat_map(|call| {
                        line.match_indices(call)
                            .map(|(at, _)| &line[at + call.len()..])
                    })
                    .map(|rest| rest.split('"').next().unwrap_or(rest))
                    .filter(|literal| literal.starts_with(|ch: char| ch.is_ascii_alphabetic()))
                    .collect()
            };
            let key = |literal: &str| literal.contains('.') && !literal.contains(' ');
            let words =
                |literal: &str| literal.contains(' ') || literal.starts_with(char::is_uppercase);
            let english = (literals(&shown).into_iter().filter(|literal| !key(literal)))
                .chain(
                    literals(&labels)
                        .into_iter()
                        .filter(|literal| words(literal)),
                )
                .chain(
                    line.trim()
                        .strip_prefix('"')
                        .and_then(|rest| rest.split('"').next()),
                )
                .filter(|literal| {
                    literal.starts_with(char::is_uppercase) || !line.trim().starts_with('"')
                });
            if english.count() > 0 {
                found.push(format!("{path}:{}: {}", no + 1, line.trim()));
            }
        }
    }
    assert!(
        found.is_empty(),
        "English outside the catalog:\n{}",
        found.join("\n")
    );
}
