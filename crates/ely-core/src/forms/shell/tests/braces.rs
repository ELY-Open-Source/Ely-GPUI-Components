//! `${...}` in double quotes, here-docs and delimiters.

/// Source, whether its last `#` comments, other comments.
pub(super) const CASES: &[(&str, bool, &[&str])] = &[
    (
        "z=abc; echo \"${u:-'$(echo hi >&2 # inner\n)'}\" # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; cat <<EOF\n${u:-'$(echo hi >&2 # inner\n)'}\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; echo \"${u:-$'$(echo hi >&2 # inner\n)'}\" # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; cat <<EOF\n${u:-$'$(echo hi >&2 # inner\n)'}\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; echo \"${u:-$\"$(echo hi >&2 # inner\n)\"}\" # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; cat <<EOF\n${u:-$\"$(echo hi >&2 # inner\n)\"}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "z=abc; echo \"${u:-\"$\"$(echo hi >&2 # inner\n)}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${u:-\"$\"$(echo hi >&2 # inner\n)}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "z=abc; echo \"${u:-$\"(echo hi >&2 # inner\n)\"}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${u:-$\"(echo hi >&2 # inner\n)\"}\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; echo \"${u:-'$\"$(echo hi >&2 # inner\n)\"'}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${u:-'$\"$(echo hi >&2 # inner\n)\"'}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "z=abc; echo \"${z#'$(echo hi >&2 # inner\n)'}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${z#'$(echo hi >&2 # inner\n)'}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "z=abc; echo \"${z#$'$(echo hi >&2 # inner\n)'}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${z#$'$(echo hi >&2 # inner\n)'}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "z=abc; echo \"${z#$\"$(echo hi >&2 # inner\n)\"}\" # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; cat <<EOF\n${z#$\"$(echo hi >&2 # inner\n)\"}\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; echo \"${z#\"$\"$(echo hi >&2 # inner\n)}\" # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; cat <<EOF\n${z#\"$\"$(echo hi >&2 # inner\n)}\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; echo \"${z#$\"(echo hi >&2 # inner\n)\"}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${z#$\"(echo hi >&2 # inner\n)\"}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "z=abc; echo \"${z#'$\"$(echo hi >&2 # inner\n)\"'}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${z#'$\"$(echo hi >&2 # inner\n)\"'}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "z=abc; echo \"${z/a/'$(echo hi >&2 # inner\n)'}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${z/a/'$(echo hi >&2 # inner\n)'}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "z=abc; echo \"${z/a/$'$(echo hi >&2 # inner\n)'}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${z/a/$'$(echo hi >&2 # inner\n)'}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "z=abc; echo \"${z/a/$\"$(echo hi >&2 # inner\n)\"}\" # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; cat <<EOF\n${z/a/$\"$(echo hi >&2 # inner\n)\"}\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; echo \"${z/a/\"$\"$(echo hi >&2 # inner\n)}\" # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; cat <<EOF\n${z/a/\"$\"$(echo hi >&2 # inner\n)}\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "z=abc; echo \"${z/a/$\"(echo hi >&2 # inner\n)\"}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${z/a/$\"(echo hi >&2 # inner\n)\"}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "z=abc; echo \"${z/a/'$\"$(echo hi >&2 # inner\n)\"'}\" # tail",
        true,
        &[],
    ),
    (
        "z=abc; cat <<EOF\n${z/a/'$\"$(echo hi >&2 # inner\n)\"'}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<EOF\n${x:-\"$\"$(echo hi # inner\n)\"\"}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a${x:-$'\\x41'}\"\n# body\na${x:-A}\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a${x:-$'\\x41'}\"\na${x:-'A'}\n# body\na${x:-A}\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a${x#$'\\x41'}\"\n# body\na${x#'A'}\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a${x:1:$'\\x41'}\"\n# body\na${x:1:A}\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a${x:-$\"b\"}\"\n# body\na${x:-b}\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a${x:-${y:-$'\\x41'}}\"\n# body\na${x:-${y:-A}}\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a${x[1+1]#$'\\x41'}\"\n# body\na${x[1+1]#A}\necho x # tail",
        true,
        &[],
    ),
    (
        "a=(abc abc abc); echo \"${a[1+1]#'$(echo hi >&2 # inner\n)'}\" # tail",
        true,
        &[],
    ),
    (
        "a=(abc abc abc); cat <<EOF\n${a[1+1]#'$(echo hi >&2 # inner\n)'}\nEOF\necho x # tail",
        true,
        &[],
    ),
];
