//! A small DOM that html5ever's tree builder fills.

use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
};

use html5ever::{
    Attribute, LocalName, ParseOpts, QualName, local_name, ns, parse_fragment,
    tendril::{StrTendril, TendrilSink},
    tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink},
};

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

struct Sink {
    nodes: RefCell<Vec<Node>>,
    names: RefCell<Vec<QualName>>,
}

impl Default for Sink {
    fn default() -> Self {
        let sink = Self {
            nodes: RefCell::new(Vec::new()),
            names: RefCell::new(Vec::new()),
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
                    return written.push_str(&text);
                }
                drop(nodes);
                let node = self.make(Data::Text(text.to_string()), unnamed());
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
