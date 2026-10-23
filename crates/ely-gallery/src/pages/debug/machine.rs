use std::rc::Rc;

use ely_gpui_component::{
    debug::{Disassembly, HexViewer, Instruction},
    theme::{ActiveTheme, Radius},
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::ui::{keep, section, set};

/// A few hundred bytes of a process's memory, the same each run.
fn memory() -> Rc<Vec<u8>> {
    let mut bytes: Vec<u8> = b"\x7fELF\x02\x01\x01\x00".to_vec();
    bytes.extend(b"palette\0accent\0fallback\0");
    bytes.extend((0u32..256).map(|ix| (ix.wrapping_mul(37) ^ (ix >> 3)) as u8));
    bytes.extend(b"Blend toward white by lift\0");
    Rc::new(bytes)
}

fn listing() -> Rc<Vec<Instruction>> {
    let at = |address: u64,
              bytes: &[u8],
              mnemonic: &'static str,
              operands: &'static str,
              comment: Option<&'static str>,
              source: Option<&'static str>| Instruction {
        address,
        bytes: bytes.to_vec(),
        mnemonic: mnemonic.into(),
        operands: operands.into(),
        comment: comment.map(Into::into),
        source: source.map(Into::into),
    };
    Rc::new(vec![
        at(
            0x1000_4a20,
            &[0x55],
            "push",
            "rbp",
            None,
            Some("pub fn lookup(&self, name: &str, lift: f32) -> Color {"),
        ),
        at(
            0x1000_4a21,
            &[0x48, 0x89, 0xe5],
            "mov",
            "rbp, rsp",
            None,
            None,
        ),
        at(
            0x1000_4a24,
            &[0x48, 0x83, 0xec, 0x30],
            "sub",
            "rsp, 0x30",
            None,
            None,
        ),
        at(
            0x1000_4a28,
            &[0x48, 0x89, 0x7d, 0xe8],
            "mov",
            "qword ptr [rbp - 0x18], rdi",
            Some("self"),
            None,
        ),
        at(
            0x1000_4a2c,
            &[0xf3, 0x0f, 0x11, 0x45, 0xe4],
            "movss",
            "dword ptr [rbp - 0x1c], xmm0",
            Some("lift"),
            None,
        ),
        at(
            0x1000_4a31,
            &[0xe8, 0x9a, 0x01, 0x00, 0x00],
            "call",
            "0x100004bd0",
            Some("BTreeMap::get"),
            Some("    let base = self.colors.get(name).copied().unwrap_or(self.fallback);"),
        ),
        at(
            0x1000_4a36,
            &[0x48, 0x85, 0xc0],
            "test",
            "rax, rax",
            None,
            None,
        ),
        at(0x1000_4a39, &[0x74, 0x12], "je", "0x100004a4d", None, None),
        at(
            0x1000_4a3b,
            &[0xf3, 0x0f, 0x10, 0x45, 0xe4],
            "movss",
            "xmm0, dword ptr [rbp - 0x1c]",
            None,
            Some("    base.lift(lift)"),
        ),
        at(
            0x1000_4a40,
            &[0xe8, 0x4b, 0x02, 0x00, 0x00],
            "call",
            "0x100004c90",
            Some("Color::lift"),
            None,
        ),
        at(
            0x1000_4a45,
            &[0x48, 0x83, 0xc4, 0x30],
            "add",
            "rsp, 0x30",
            None,
            None,
        ),
        at(0x1000_4a49, &[0x5d], "pop", "rbp", None, None),
        at(0x1000_4a4a, &[0xc3], "ret", "", None, Some("}")),
    ])
}

pub fn machine(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let picked = keep("debug-byte", || 8usize, window, cx);
    let marks = keep("debug-marks", || vec![5usize], window, cx);
    let (now_picked, now_marks) = (*picked.read(cx), marks.read(cx).clone());
    let (pick, toggle) = (picked.clone(), marks.clone());
    let theme = cx.theme();
    let frame = || {
        div()
            .w(px(840.))
            .p_2()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
    };
    section(
        "MemoryViewer / HexViewer / Disassembly",
        "Memory as a hex dump, sixteen bytes a row in fours beside the same bytes as text; the byte under the pointer lights on both sides and the picked one reads below as numbers. Machine code with its source lines, registers and numbers colored, the current instruction marked; press the gutter to set a breakpoint.",
        cx,
    )
    .child(
        frame().h(px(220.)).child(
            HexViewer::new("debug-hex", memory())
                .base(0x7ffd_5a20_1000)
                .selected(now_picked..(now_picked + 4).min(memory().len()))
                .on_select(move |at, _, cx| set(&pick, at, cx)),
        ),
    )
    .child(
        frame().h(px(300.)).child(
            Disassembly::new("debug-disasm", listing())
                .current(8)
                .breakpoints(now_marks)
                .on_breakpoint(move |ix, _, cx| {
                    let mut next = toggle.read(cx).clone();
                    match next.iter().position(|mark| *mark == ix) {
                        Some(at) => drop(next.remove(at)),
                        None => next.push(ix),
                    }
                    set(&toggle, next, cx)
                }),
        ),
    )
}
