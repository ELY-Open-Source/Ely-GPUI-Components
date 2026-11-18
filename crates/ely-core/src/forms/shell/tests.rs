use super::comments;

mod braces;

/// Source, whether its last `#` comments, other comments.
const CASES: &[(&str, bool, &[&str])] = &[
    ("echo x;# return 42", true, &[]),
    ("echo x&# return 42", true, &[]),
    ("(echo x)# return 4", true, &[]),
    ("(echo $(a))# return 4", true, &[]),
    ("(case a in a) echo y;; esac)# return 4", true, &[]),
    ("case a in a)# return 42", true, &[]),
    ("echo x\\\\;# return 4", true, &[]),
    ("echo x # return 42", true, &[]),
    ("echo foo#bar 42", false, &[]),
    ("echo x\\;# return 42", false, &[]),
    ("echo x\\&# return 42", false, &[]),
    ("echo x\\ # return 42", false, &[]),
    ("echo x\\\\\\;# return 42", false, &[]),
    ("echo \"a;\"# return 42", false, &[]),
    ("echo 1 $(echo a)# return 42", false, &[]),
    ("echo 2 $((1+2))# return 42", false, &[]),
    ("cat <(echo 4 p)# return 42", false, &[]),
    ("tee >(cat)# return 42", false, &[]),
    (
        "echo 5 $(case a in a) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo 6 $(case a in (a) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo 7 $(case a in a) case b in b) echo z;; esac;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo 8 $(case a in @(a|b)) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo 9 $(if true; then case a in a) echo y;& b) echo w;; esac; fi)# return 42",
        false,
        &[],
    ),
    ("echo 10 $(echo case)# return 42", false, &[]),
    (
        "echo A $(case case in a) echo n;; case) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo B $(case case in b|case) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo C $(case case in (case) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo D $(case a in a) if true; then echo y; fi esac)# return 42",
        false,
        &[],
    ),
    (
        "echo E $(case a in a) { echo y; } esac)# return 42",
        false,
        &[],
    ),
    (
        "echo F $(case a in a) for i in 1; do echo $i; done esac)# return 42",
        false,
        &[],
    ),
    (
        "echo G $(case a in x-esac) echo n;; a) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo H $(case a in a) echo done esac;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo I $(case case in @(case|foo)) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo J $(case esac in (esac) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo K $(case esac in foo|esac) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo M $(case a in @(b|(c))) echo n;; a) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo N $(case a in a) echo y;;\nesac)# return 42",
        false,
        &[],
    ),
    ("echo O $(case a in esac)# return 42", false, &[]),
    (
        "echo Q $(case a in @(b)|esac) echo n;; a) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo R $(case esac in 1|esac) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo S $(case esac in \"foo\"|esac) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo T $(case esac in x-a|esac) echo y;; esac)# return 42",
        false,
        &[],
    ),
    (
        "echo U $(case esac in ?|esac) echo y;; esac)# return 42",
        false,
        &[],
    ),
    ("echo \\(x\\)# return 42", false, &[]),
    ("echo V $(case a in # c\nesac)# return 42", false, &["# c"]),
    (
        "echo W $(case a in a) echo y;;& # c\n esac)# return 42",
        false,
        &["# c"],
    ),
    ("echo X $(case a in \\\nesac)# return 42", false, &[]),
    (
        "echo Y $(case a in a) echo y;; \\\nesac)# return 42",
        false,
        &[],
    ),
    (
        "echo Z $(case a in a) echo y;;& \\\n# c\nesac)# return 42",
        false,
        &["# c"],
    ),
    ("echo A $(case a in \\\nesac)#tail", false, &[]),
    (
        "echo y $(case a in a) echo y;; \\\n# note\nesac)#tail",
        false,
        &["# note"],
    ),
    (
        "echo y $(case a in a) echo y;;& \\\n# note\nesac)#tail",
        false,
        &["# note"],
    ),
    ("cat <<EOF\n# body\nEOF\necho x # return 42", true, &[]),
    ("cat <<-EOF\n\t# body\n\tEOF\necho x # return 42", true, &[]),
    ("cat <<'EOF'\n# $(body\nEOF\necho x # return 42", true, &[]),
    ("cat <<E\\OF\n# body\nEOF\necho x # return 42", true, &[]),
    (
        "cat <<EOF | cat # c\n# body\nEOF\necho x # return 42",
        true,
        &["# c"],
    ),
    ("echo $(cat <<EOF\n)# body\nEOF\n)# return 42", false, &[]),
    ("echo 'a\n# in' # return 42", true, &[]),
    ("echo \"a\n#in\" # return 42", true, &[]),
    ("echo \"$(echo \"#q\")\"# return 42", false, &[]),
    ("echo a \\\n# return 42", true, &[]),
    ("echo a\\\n# return 42", false, &[]),
    ("x=abc; echo ${#x} # return 42", true, &[]),
    ("x=abc; echo ${x#a}# return 42", false, &[]),
    ("echo $((2#11)) # return 42", true, &[]),
    ("echo `echo a`# return 42", false, &[]),
    ("echo $'a\\'b'# return 42", false, &[]),
    ("f() { echo y; }; f # return 42", true, &[]),
    ("echo <<<x # return 42", true, &[]),
    (
        "echo `echo hi # inner\nprintf y` # tail",
        true,
        &["# inner"],
    ),
    (
        "echo \"`echo hi # inner\nprintf y`\" # tail",
        true,
        &["# inner"],
    ),
    ("cat <<'E\\OF'\n# body\nE\\OF\necho x # tail", true, &[]),
    ("cat <<$'EOF'\n# body\nEOF\necho x # tail", true, &[]),
    ("cat <<E\\\nOF\n# body\nEOF\necho x # tail", true, &[]),
    ("cat <<\"E\\\"OF\"\n# body\nE\"OF\necho x # tail", true, &[]),
    ("echo $(c\\\nase a in a) echo y;; esac)#tail", false, &[]),
    ("echo $(case a i\\\nn a) echo y;; esac)#tail", false, &[]),
    ("echo $(case a in a) echo y;; es\\\nac)#tail", false, &[]),
    (
        "echo $(time -p case a in a) echo y;; esac)#tail",
        false,
        &[],
    ),
    (
        "echo $(time -p -- case a in a) echo y;; esac)#tail",
        false,
        &[],
    ),
    ("echo $(time case a in a) echo y;; esac)#tail", false, &[]),
    (
        "echo $(coproc N case a in a) echo y;; esac)#tail",
        false,
        &[],
    ),
    ("coproc N { echo y; }; wait # tail", true, &[]),
    (
        "echo $(function f { case a in a) echo y;; esac; }; f)#tail",
        false,
        &[],
    ),
    (
        "echo $(function f() { case a in a) echo y;; esac; }; f)#tail",
        false,
        &[],
    ),
    (
        "echo $(function f\n{ case a in a) echo y;; esac; }; f)#tail",
        false,
        &[],
    ),
    ("cat <<$'E\\x4fF'\n# body\nEOF\necho x # tail", true, &[]),
    ("cat <<$'E\\117F'\n# body\nEOF\necho x # tail", true, &[]),
    ("cat <<$'E\\u004fF'\n# body\nEOF\necho x # tail", true, &[]),
    ("cat <<$'E\\qF'\n# body\nE\\qF\necho x # tail", true, &[]),
    (
        "echo ${x:-`echo hi # inner\nprintf y`} # tail",
        true,
        &["# inner"],
    ),
    (
        "echo $(case a in @(a|$(echo b # inner\nprintf c))) echo y;; esac)#tail",
        false,
        &["# inner"],
    ),
    (
        "echo $(case a in @(a|`echo b # inner\nprintf c`)) echo y;; esac)#tail",
        false,
        &["# inner"],
    ),
    (
        "echo $((1 + `echo 2 # inner\n`)) # tail",
        true,
        &["# inner"],
    ),
    (
        "cat <<EOF\n$(echo a # inner\n)\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "cat <<EOF\n`echo a # inner\n`\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "cat <<'EOF'\n$(echo a # body\n)\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<EOF\n\\$(echo a # body)\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<E\\\nOF\n$(echo a # inner\n)\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "cat <<E\\\nOF\n`echo a # inner\n`\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "cat <<\"E\\\nOF\"\n$(echo a # body\n)\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "echo \"$'$(echo hi # inner\n)'\" # tail",
        true,
        &["# inner"],
    ),
    (
        "cat <<EOF\n$'$(echo hi # inner\n)'\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "cat <<EOF\n$\"$(echo hi # inner\n)\"\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    ("echo \"a$\"b # tail", true, &[]),
    (
        "echo \"${x:-$'$(echo hi # inner\n)'}\" # tail",
        true,
        &["# inner"],
    ),
    (
        "echo \"${x:-'$(echo hi # inner\n)'}\" # tail",
        true,
        &["# inner"],
    ),
    (
        "cat <<EOF\n${x:-'$(echo hi # inner\n)'}\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    ("echo \"${x:-'a}b'} # body\" # tail", true, &[]),
    (
        "cat <<EOF\n${x:-\"a}b\"} # body\nEOF\necho x # tail",
        true,
        &[],
    ),
    ("echo ${x:-$'$(echo hi # body\n)'} # tail", true, &[]),
    (
        "cat <<$(: \"x\")\n# body\n$(: \"x\")\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<$(: \"x\")\n$(echo a # inner\n)\n$(: \"x\")\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "cat <<`: \"x\"`\n# body\n`: \"x\"`\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<${x:-\"y\"}\n# body\n${x:-\"y\"}\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<a$(: \"x\")'b'\n# body\na$(: x)b\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a\"$(: $'\\x41')\n# body\na$(: A)\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<$(: $'\\x41')\n# body\n$(: 'A')\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a$(: $'\\x41')\"\n# body\na$(: 'A')\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a\"`: $'\\x41'`\n# body\na`: $\\x41`\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<$(: \"x\\\ny\")\n# body\n$(: \"xy\")\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<\"a\"`: 'x\\\ny'`\n# body\na`: xy`\necho x # tail",
        true,
        &[],
    ),
    ("cat <<EOF\nE\\\nOF\necho x # tail", true, &[]),
    ("cat <<'EOF'\nE\\\nOF\nEOF\necho x # tail", true, &[]),
    ("cat <<EOF\nx\\\\\nEOF\necho x # tail", true, &[]),
    ("cat <<EOF\nx\\\\\\\nEOF\nEOF\necho x # tail", true, &[]),
    ("cat <<-EOF\n\tE\\\n\tOF\nEOF\necho x # tail", true, &[]),
    (
        "cat <<`: x\\\ny`\n# body\n`: x\\\ny`\necho x # tail",
        true,
        &[],
    ),
    ("echo \"${x:-'}\"'}\" # tail", true, &[]),
    ("echo \"${x:-'}'\"y\"}\" # tail", true, &[]),
    ("cat <<`: x\\\\\ny`\n`: x\\y`\necho x # tail", false, &[]),
    ("cat <<$(: $'\\'')\n$(: \\')\necho x # tail", true, &[]),
    (
        "cat <<$(: $'\\'')\n$(: ''\\''')\n# body\n$(: \\')\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<$(: $'a\\'b')\n$(: 'a'\\''b')\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<EOF\n${x:-$\"$(echo hi # inner\n)\"}\nEOF\necho x # tail",
        true,
        &[],
    ),
    (
        "cat <<EOF\n${x:-$\"(echo hi # inner\n)\"}\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "cat <<EOF\n${x:-$\"`echo hi # inner\n`\"}\nEOF\necho x # tail",
        true,
        &["# inner"],
    ),
    (
        "echo \"${x:-$\"$(echo hi # inner\n)\"}\" # tail",
        true,
        &["# inner"],
    ),
];

#[test]
fn comments_start_where_bash_starts_them() {
    for (source, tail, inner) in CASES.iter().chain(braces::CASES) {
        let mut want: Vec<usize> = inner
            .iter()
            .map(|needle| source.find(needle).expect("the needle"))
            .collect();
        if *tail {
            want.push(source.rfind('#').expect("a tail"));
        }
        want.sort_unstable();
        assert_eq!(comments(source), want, "{source:?}");
    }
}
