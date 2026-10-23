use ely_gpui_component::documents::Latex;
use gpui::{App, IntoElement, ParentElement, Styled};

use crate::ui::{code, section, specimen, specimens};

pub fn latex(cx: &App) -> impl IntoElement + use<> {
    let math =
        |id: &'static str, source: &str| Latex::parse(id, source).expect("gallery formula parses");
    section(
        "Latex / MathInline",
        "TeX typeset by RaTeX, in the text's color.",
        cx,
    )
    .child(
        specimens()
            .gap_10()
            .child(specimen("euler", math("euler", r"e^{i\pi} + 1 = 0"), cx))
            .child(specimen(
                "sum",
                math("sum", r"\sum_{k=1}^{n} k = \frac{n(n+1)}{2}"),
                cx,
            ))
            .child(specimen(
                "root",
                math("root", r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}"),
                cx,
            )),
    )
    .child(code("Latex::parse(\"euler\", r\"e^{i\\pi} + 1 = 0\")?", cx))
}
