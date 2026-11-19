//! A small DOM that html5ever's tree builder fills.

use std::{
    borrow::Cow,
    cell::{Cell, Ref, RefCell},
};

use html5ever::{
    Attribute, LocalName, ParseOpts, Parser, QualName, local_name, ns, parse_fragment,
    tendril::{StrTendril, TendrilSink},
    tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink},
};

use super::html::veils;

/// What a node is.
#[derive(Debug)]
pub(crate) enum Data {
    Document,
    Element {
        attributes: Vec<Attribute>,
    },
    Text(String),
    /// Comments and the like: they draw nothing.
    Other,
}

#[derive(Debug)]
pub(crate) struct Node {
    pub data: Data,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

/// The parsed tree; node 0 is the document.
#[derive(Debug)]
pub(crate) struct Dom {
    pub nodes: Vec<Node>,
    pub names: Vec<QualName>,
    /// Some element may hide or replace what it holds.
    pub veiled: bool,
}

impl Dom {
    /// An element's lowercase local name; empty for other nodes.
    pub(crate) fn name(&self, node: usize) -> &str {
        &self.names[node].local
    }

    pub(crate) fn attribute(&self, node: usize, name: &str) -> Option<&str> {
        match &self.nodes[node].data {
            Data::Element { attributes } => attributes
                .iter()
                .find(|attribute| &*attribute.name.local == name)
                .map(|attribute| &*attribute.value),
            _ => None,
        }
    }

    /// Whether element `node` carries the attribute `name`.
    pub(crate) fn has(&self, node: usize, name: &LocalName) -> bool {
        match &self.nodes[node].data {
            Data::Element { attributes } => attributes.iter().any(|it| it.name.local == *name),
            _ => false,
        }
    }

    /// The fragment's top nodes, under html5ever's `html` element.
    pub(crate) fn top(&self) -> &[usize] {
        match self.nodes[0].children.first() {
            Some(&html) => &self.nodes[html].children,
            None => &[],
        }
    }
}

/// `html` parsed as the children of a `context` element.
pub(crate) fn fragment(html: &str, context: &str) -> Dom {
    if context == "body"
        && let Some(dom) = nested(html)
    {
        return dom;
    }
    let context = QualName::new(None, ns!(html), LocalName::from(context));
    parse_fragment(
        Sink::default(),
        ParseOpts::default(),
        context,
        Vec::new(),
        false,
    )
    .one(html)
}

/// HTML fed in pieces, its tree built as it goes.
pub(crate) struct Stream(Parser<Sink>);

impl Stream {
    /// A stream inside `open`, the markup the drawing wraps.
    pub(crate) fn within(open: &str) -> Self {
        let context = element("body");
        let mut stream = Self(parse_fragment(
            Sink::default(),
            ParseOpts::default(),
            context,
            Vec::new(),
            false,
        ));
        stream.feed(open.into());
        stream
    }

    pub(crate) fn feed(&mut self, html: String) {
        self.0.process(html.into());
    }

    /// Whether the last text lies inside an element named `names`.
    pub(crate) fn text_within(&self, names: &[&str]) -> bool {
        let sink = &self.0.tokenizer.sink.sink;
        let (nodes, named) = (sink.nodes.borrow(), sink.names.borrow());
        let mut at = nodes[sink.last.get()].parent;
        while let Some(node) = at {
            if names.contains(&&*named[node].local) {
                return true;
            }
            at = nodes[node].parent;
        }
        false
    }
}

/// Well-nested markdown markup as html5ever builds it; else `None`.
fn nested(html: &str) -> Option<Dom> {
    if html.contains('\0') {
        return None;
    }
    // HTML's input stream reads line endings as line feeds.
    let html = match html.contains('\r') {
        true => Cow::Owned(html.replace("\r\n", "\n").replace('\r', "\n")),
        false => Cow::Borrowed(html),
    };
    let sink = Sink::default();
    let root = sink.make(
        Data::Element {
            attributes: Vec::new(),
        },
        element("html"),
    );
    sink.append(&0, NodeOrText::AppendNode(root));
    let mut open: Vec<(usize, &str)> = Vec::new();
    let mut rest = &*html;
    while !rest.is_empty() {
        let (parent, name) = open
            .last()
            .map_or((root, None), |(node, name)| (*node, Some(*name)));
        let Some(tag) = rest.strip_prefix('<') else {
            let end = rest.find('<').unwrap_or(rest.len());
            // Text in a table's rows moves out of it.
            if matches!(name, Some("table" | "tbody" | "tr")) {
                return None;
            }
            let text = sink.make(Data::Text(decoded(&rest[..end])?), unnamed());
            sink.append(&parent, NodeOrText::AppendNode(text));
            rest = &rest[end..];
            continue;
        };
        let end = tag.find('>')?;
        rest = &tag[end + 1..];
        if let Some(closing) = tag[..end].strip_prefix('/') {
            match open.pop() {
                Some((_, opened)) if opened == closing => continue,
                _ => return None,
            }
        }
        let (tag, written) = tag[..end].split_once(' ').unwrap_or((&tag[..end], ""));
        if !fits(tag, name) || (tag == "a" && open.iter().any(|(_, name)| *name == "a")) {
            return None;
        }
        let attributes = attributes(written)?;
        // `fits` admits no element that hides; `hidden` still does.
        if attributes
            .iter()
            .any(|it| it.name.local == local_name!("hidden"))
        {
            sink.veiled.set(true);
        }
        let node = sink.make(Data::Element { attributes }, element(tag));
        sink.append(&parent, NodeOrText::AppendNode(node));
        if !matches!(tag, "br" | "img") {
            open.push((node, tag));
        }
    }
    open.is_empty().then(|| sink.finish())
}

/// Whether markdown's `name` opens in `parent` as written.
fn fits(name: &str, parent: Option<&str>) -> bool {
    match name {
        "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "table" => parent.is_none(),
        "tbody" => parent == Some("table"),
        "tr" => parent == Some("tbody"),
        "td" => parent == Some("tr"),
        "strong" | "em" | "del" | "a" | "code" | "span" | "br" | "img" => {
            !matches!(parent, Some("table" | "tbody" | "tr"))
        }
        _ => false,
    }
}

fn element(name: &str) -> QualName {
    QualName::new(None, ns!(html), LocalName::from(name))
}

/// Attributes written ` name` or ` name="value"`, each once.
fn attributes(mut rest: &str) -> Option<Vec<Attribute>> {
    let mut out: Vec<Attribute> = Vec::new();
    while !rest.is_empty() {
        let end = rest.find([' ', '=']).unwrap_or(rest.len());
        let name = &rest[..end];
        let odd = |ch: char| ch.is_ascii_uppercase() || "\"'<>/\t\n\x0c".contains(ch);
        if name.is_empty() || name.contains(odd) || out.iter().any(|a| &*a.name.local == name) {
            return None;
        }
        rest = &rest[end..];
        let value = match rest.strip_prefix("=\"") {
            Some(quoted) => {
                let close = quoted.find('"')?;
                rest = &quoted[close + 1..];
                decoded(&quoted[..close])?
            }
            None => String::new(),
        };
        if !rest.is_empty() {
            rest = rest.strip_prefix(' ').filter(|rest| !rest.is_empty())?;
        }
        out.push(Attribute {
            name: QualName::new(None, ns!(), LocalName::from(name)),
            value: value.into(),
        });
    }
    Some(out)
}

/// Text with the references `escaped` writes read; `None` at others.
fn decoded(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let (reference, ch) = [
            ("&amp;", '&'),
            ("&lt;", '<'),
            ("&gt;", '>'),
            ("&quot;", '"'),
        ]
        .into_iter()
        .find(|(reference, _)| rest.starts_with(reference))?;
        out.push(ch);
        rest = &rest[reference.len()..];
    }
    out.push_str(rest);
    Some(out)
}

struct Sink {
    nodes: RefCell<Vec<Node>>,
    names: RefCell<Vec<QualName>>,
    /// The node text last went to.
    last: Cell<usize>,
    veiled: Cell<bool>,
}

impl Default for Sink {
    fn default() -> Self {
        let sink = Self {
            nodes: RefCell::new(Vec::new()),
            names: RefCell::new(Vec::new()),
            last: Cell::new(0),
            veiled: Cell::new(false),
        };
        sink.make(Data::Document, unnamed());
        sink
    }
}

fn unnamed() -> QualName {
    QualName::new(None, ns!(), local_name!(""))
}

impl Sink {
    fn make(&self, data: Data, name: QualName) -> usize {
        let mut nodes = self.nodes.borrow_mut();
        nodes.push(Node {
            data,
            parent: None,
            children: Vec::new(),
        });
        self.names.borrow_mut().push(name);
        nodes.len() - 1
    }

    fn detach(&self, node: usize) {
        let mut nodes = self.nodes.borrow_mut();
        if let Some(parent) = nodes[node].parent.take() {
            nodes[parent].children.retain(|&child| child != node);
        }
    }

    /// Inserts `child` at `at`; text merges with text beside it.
    fn insert(&self, parent: usize, at: usize, child: NodeOrText<usize>) {
        match child {
            NodeOrText::AppendText(text) => {
                let mut nodes = self.nodes.borrow_mut();
                let before = at.checked_sub(1).map(|ix| nodes[parent].children[ix]);
                if let Some(before) = before
                    && let Data::Text(written) = &mut nodes[before].data
                {
                    self.last.set(before);
                    return written.push_str(&text);
                }
                drop(nodes);
                let node = self.make(Data::Text(text.to_string()), unnamed());
                self.last.set(node);
                let mut nodes = self.nodes.borrow_mut();
                nodes[node].parent = Some(parent);
                nodes[parent].children.insert(at, node);
            }
            NodeOrText::AppendNode(node) => {
                self.detach(node);
                let mut nodes = self.nodes.borrow_mut();
                let at = at.min(nodes[parent].children.len());
                nodes[node].parent = Some(parent);
                nodes[parent].children.insert(at, node);
            }
        }
    }
}

impl TreeSink for Sink {
    type Handle = usize;
    type Output = Dom;
    type ElemName<'a> = Ref<'a, QualName>;

    fn finish(self) -> Dom {
        Dom {
            nodes: self.nodes.into_inner(),
            names: self.names.into_inner(),
            veiled: self.veiled.get(),
        }
    }

    fn parse_error(&self, _: Cow<'static, str>) {}

    fn get_document(&self) -> usize {
        0
    }

    fn elem_name<'a>(&'a self, target: &'a usize) -> Ref<'a, QualName> {
        Ref::map(self.names.borrow(), |names| &names[*target])
    }

    fn create_element(
        &self,
        name: QualName,
        attributes: Vec<Attribute>,
        flags: ElementFlags,
    ) -> usize {
        if veils(&name.local, &attributes) {
            self.veiled.set(true);
        }
        let element = self.make(Data::Element { attributes }, name);
        // A template's contents hang under it, as a fragment.
        if flags.template {
            let contents = self.make(Data::Document, unnamed());
            self.insert(element, 0, NodeOrText::AppendNode(contents));
        }
        element
    }

    fn create_comment(&self, _: StrTendril) -> usize {
        self.make(Data::Other, unnamed())
    }

    fn create_pi(&self, _: StrTendril, _: StrTendril) -> usize {
        self.make(Data::Other, unnamed())
    }

    fn append(&self, parent: &usize, child: NodeOrText<usize>) {
        let at = self.nodes.borrow()[*parent].children.len();
        self.insert(*parent, at, child);
    }

    fn append_based_on_parent_node(
        &self,
        element: &usize,
        previous: &usize,
        child: NodeOrText<usize>,
    ) {
        let placed = self.nodes.borrow()[*element].parent.is_some();
        match placed {
            true => self.append_before_sibling(element, child),
            false => self.append(previous, child),
        }
    }

    fn append_doctype_to_document(&self, _: StrTendril, _: StrTendril, _: StrTendril) {}

    fn get_template_contents(&self, target: &usize) -> usize {
        self.nodes.borrow()[*target].children[0]
    }

    fn same_node(&self, x: &usize, y: &usize) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, _: QuirksMode) {}

    fn append_before_sibling(&self, sibling: &usize, child: NodeOrText<usize>) {
        let nodes = self.nodes.borrow();
        let Some(parent) = nodes[*sibling].parent else {
            return;
        };
        let at = nodes[parent]
            .children
            .iter()
            .position(|node| node == sibling);
        drop(nodes);
        if let Some(at) = at {
            self.insert(parent, at, child);
        }
    }

    fn add_attrs_if_missing(&self, target: &usize, added: Vec<Attribute>) {
        if let Data::Element { attributes } = &mut self.nodes.borrow_mut()[*target].data {
            for attribute in added {
                if !attributes.iter().any(|kept| kept.name == attribute.name) {
                    attributes.push(attribute);
                }
            }
        }
    }

    fn remove_from_parent(&self, target: &usize) {
        self.detach(*target);
    }

    fn reparent_children(&self, node: &usize, new_parent: &usize) {
        let children = std::mem::take(&mut self.nodes.borrow_mut()[*node].children);
        for child in children {
            self.nodes.borrow_mut()[child].parent = None;
            self.append(new_parent, NodeOrText::AppendNode(child));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Data, fragment};

    /// Text under each element, by element name.
    fn shape(html: &str) -> String {
        let dom = fragment(html, "p");
        let mut out = String::new();
        fn walk(dom: &super::Dom, node: usize, out: &mut String) {
            match &dom.nodes[node].data {
                Data::Text(text) => out.push_str(text),
                Data::Element { .. } => {
                    out.push_str(&format!("<{}>", dom.name(node)));
                    for &child in &dom.nodes[node].children {
                        walk(dom, child, out);
                    }
                    out.push_str(&format!("</{}>", dom.name(node)));
                }
                _ => {}
            }
        }
        for &node in dom.top() {
            walk(&dom, node, &mut out);
        }
        out
    }

    #[test]
    fn html5ever_repairs_misnested_formatting() {
        // The adoption agency, as browsers run it.
        assert_eq!(shape("<b>1<i>2</b>3</i>"), "<b>1<i>2</i></b><i>3</i>");
        assert_eq!(shape("a<!-- c -->b"), "ab");
        assert_eq!(shape("<a href=x>1<a href=y>2</a>"), "<a>1</a><a>2</a>");
    }
}
