use std::rc::Rc;

use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window};

use crate::{
    canvas::{Edge, Node, NodeGraph, Port, Viewport},
    lists::{Tree, TreeNode},
    primitives::IconName,
};

/// The key a field takes part in: its table's own, or one that points at another table's field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldKey {
    /// Its table's own key.
    Primary,
    /// A key that points at another table's field.
    Foreign {
        /// The table it points at.
        table: SharedString,
        /// The field it points at.
        field: SharedString,
    },
}

/// A field of a table: its name and type, whether it may be empty, the key it takes part in, and its default.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    /// Its name.
    pub name: SharedString,
    /// Its type, as the database writes it.
    pub ty: SharedString,
    /// Whether it may be empty.
    pub nullable: bool,
    /// The key it takes part in.
    pub key: Option<FieldKey>,
    /// Its default.
    pub default: Option<SharedString>,
}

impl Field {
    /// A field named `name` of type `ty`: not empty, no key, no default.
    pub fn new(name: impl Into<SharedString>, ty: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            ty: ty.into(),
            nullable: false,
            key: None,
            default: None,
        }
    }

    /// Marks it as one that may be empty.
    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    /// Makes it its table's own key.
    pub fn primary(mut self) -> Self {
        self.key = Some(FieldKey::Primary);
        self
    }

    /// Points at `field` of `table`.
    pub fn refers(
        mut self,
        table: impl Into<SharedString>,
        field: impl Into<SharedString>,
    ) -> Self {
        self.key = Some(FieldKey::Foreign {
            table: table.into(),
            field: field.into(),
        });
        self
    }

    /// Its default.
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Its type with the key it takes part in.
    fn typed(&self) -> SharedString {
        match &self.key {
            Some(FieldKey::Primary) => format!("{} · PK", self.ty).into(),
            Some(FieldKey::Foreign { .. }) => format!("{} · FK", self.ty).into(),
            None => self.ty.clone(),
        }
    }
}

/// A table, or a view, and its fields.
#[derive(Clone, Debug, PartialEq)]
pub struct DbTable {
    /// Its name.
    pub name: SharedString,
    /// Whether it is a view.
    pub view: bool,
    /// Its fields, in order.
    pub fields: Vec<Field>,
}

impl DbTable {
    /// A table named `name` of `fields`.
    pub fn new(name: impl Into<SharedString>, fields: impl IntoIterator<Item = Field>) -> Self {
        Self {
            name: name.into(),
            view: false,
            fields: fields.into_iter().collect(),
        }
    }

    /// Marks it as a view.
    pub fn view(mut self) -> Self {
        self.view = true;
        self
    }
}

/// A schema of a database, and its tables and views.
#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    /// Its name.
    pub name: SharedString,
    /// Its tables and views.
    pub tables: Vec<DbTable>,
}

type OnKeys = Rc<dyn Fn(&[SharedString], &mut Window, &mut App)>;

/// Schemas, their tables and views, and each one's fields with its type; keys are marked. A row's key is its path: `schema`, `schema.table` or `schema.table.field`. It fills its box, which the host sizes.
///
/// ```
/// use ely_devtools::{DbTable, Field, Schema, SchemaTree};
///
/// let users = DbTable::new("users", [Field::new("id", "int").primary(), Field::new("email", "text")]);
/// let tree = SchemaTree::new("schema", [Schema { name: "public".into(), tables: vec![users] }])
///     .on_select(|_paths, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct SchemaTree {
    id: ElementId,
    schemas: Vec<Schema>,
    open: Vec<SharedString>,
    selected: Vec<SharedString>,
    on_select: Option<OnKeys>,
}

impl SchemaTree {
    /// A tree under `id` of `schemas`, every one shut, nothing selected.
    pub fn new(id: impl Into<ElementId>, schemas: impl IntoIterator<Item = Schema>) -> Self {
        Self {
            id: id.into(),
            schemas: schemas.into_iter().collect(),
            open: Vec::new(),
            selected: Vec::new(),
            on_select: None,
        }
    }

    /// The paths open at first.
    pub fn open(mut self, keys: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.open = keys.into_iter().map(Into::into).collect();
        self
    }

    /// The rows selected now, by path.
    pub fn selected(mut self, keys: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.selected = keys.into_iter().map(Into::into).collect();
        self
    }

    /// Gets the paths selected after each press.
    pub fn on_select(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SchemaTree {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let nodes = self.schemas.iter().map(|schema| {
            TreeNode::new(schema.name.clone(), schema.name.clone())
                .icon(IconName::Database)
                .children(schema.tables.iter().map(|table| {
                    let path = format!("{}.{}", schema.name, table.name);
                    TreeNode::new(path.clone(), table.name.clone())
                        .icon(if table.view {
                            IconName::Eye
                        } else {
                            IconName::Table
                        })
                        .children(table.fields.iter().map(|field| {
                            let icon = match field.key {
                                Some(FieldKey::Primary) => IconName::Key,
                                Some(FieldKey::Foreign { .. }) => IconName::Link,
                                None => IconName::Columns2,
                            };
                            TreeNode::new(format!("{path}.{}", field.name), field.name.clone())
                                .icon(icon)
                                .note(field.ty.clone())
                        }))
                }))
        });
        let tree = Tree::new(self.id, nodes)
            .open(self.open)
            .selected(self.selected)
            .size_full();
        match self.on_select {
            Some(on_select) => tree.on_select(move |keys, window, cx| on_select(keys, window, cx)),
            None => tree,
        }
    }
}

type OnNudge = Rc<dyn Fn(&SharedString, (f32, f32), &mut Window, &mut App)>;
type OnViewport = Rc<dyn Fn(Viewport, &mut Window, &mut App)>;

/// Tables as boxes of their fields, names down the left and types down the right, with a line from each foreign key to the field it points at. A table's title drags it; the owner keeps where each one sits. A key that points at no field fails loud.
///
/// ```
/// use ely_canvas::Viewport;
/// use ely_devtools::{DbTable, ERDiagram, Field};
///
/// let users = DbTable::new("users", [Field::new("id", "int").primary()]);
/// let posts = DbTable::new("posts", [Field::new("id", "int").primary(), Field::new("author", "int").refers("users", "id")]);
/// let diagram = ERDiagram::new("er", Viewport::new(0.0, 0.0, 1.0), [users, posts], [("users", (0.0, 0.0)), ("posts", (260.0, 0.0))]);
/// ```
#[derive(IntoElement)]
pub struct ERDiagram {
    id: ElementId,
    viewport: Viewport,
    tables: Vec<DbTable>,
    places: Vec<(SharedString, (f32, f32))>,
    on_move: Option<OnNudge>,
    on_viewport: Option<OnViewport>,
}

impl ERDiagram {
    /// `places` sets where each table sits, in canvas units, by name.
    pub fn new(
        id: impl Into<ElementId>,
        viewport: Viewport,
        tables: impl IntoIterator<Item = DbTable>,
        places: impl IntoIterator<Item = (impl Into<SharedString>, (f32, f32))>,
    ) -> Self {
        Self {
            id: id.into(),
            viewport,
            tables: tables.into_iter().collect(),
            places: places
                .into_iter()
                .map(|(name, at)| (name.into(), at))
                .collect(),
            on_move: None,
            on_viewport: None,
        }
    }

    /// Gets a table's name and how far it was dragged, in canvas units.
    pub fn on_move(
        mut self,
        handler: impl Fn(&SharedString, (f32, f32), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_move = Some(Rc::new(handler));
        self
    }

    /// Gets the viewport after each pan and zoom.
    pub fn on_viewport(
        mut self,
        handler: impl Fn(Viewport, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_viewport = Some(Rc::new(handler));
        self
    }
}

/// The diagram's nodes and wires: a node per table and a wire per foreign key.
pub(crate) fn diagram(
    tables: &[DbTable],
    places: &[(SharedString, (f32, f32))],
) -> (Vec<Node>, Vec<Edge>) {
    let nodes = tables
        .iter()
        .filter_map(|table| {
            let Some((_, at)) = places.iter().find(|(name, _)| *name == table.name) else {
                log::error!("ER diagram: no place for {}; left out", table.name);
                return None;
            };
            let at = *at;
            Some(table.fields.iter().fold(
                Node::new(table.name.clone(), table.name.clone(), at),
                |node, field| {
                    node.input(Port::new(field.name.clone(), field.name.clone(), "field"))
                        .output(Port::new(field.name.clone(), field.typed(), "field"))
                },
            ))
        })
        .collect();
    let edges = tables
        .iter()
        .flat_map(|table| {
            table
                .fields
                .iter()
                .filter_map(move |field| match &field.key {
                    Some(FieldKey::Foreign {
                        table: target,
                        field: to,
                    }) => Some(Edge::new(
                        (table.name.clone(), field.name.clone()),
                        (target.clone(), to.clone()),
                    )),
                    _ => None,
                })
        })
        .collect();
    (nodes, edges)
}

impl RenderOnce for ERDiagram {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (nodes, edges) = diagram(&self.tables, &self.places);
        let mut graph = NodeGraph::new(self.id, self.viewport, nodes, edges).fixed_wires();
        if let Some(on_move) = self.on_move {
            graph = graph.on_move(move |key, by, window, cx| on_move(key, by, window, cx));
        }
        if let Some(on_viewport) = self.on_viewport {
            graph = graph.on_viewport(move |view, window, cx| on_viewport(view, window, cx));
        }
        graph
    }
}

#[cfg(test)]
mod tests {
    use super::{DbTable, Field, diagram};

    #[test]
    fn a_foreign_key_draws_a_wire_to_the_field_it_points_at() {
        let tables = [
            DbTable::new(
                "users",
                [
                    Field::new("id", "int").primary(),
                    Field::new("email", "text"),
                ],
            ),
            DbTable::new(
                "orders",
                [
                    Field::new("id", "int").primary(),
                    Field::new("user_id", "int").refers("users", "id"),
                ],
            ),
        ];
        let (nodes, edges) = diagram(
            &tables,
            &[
                ("users".into(), (0.0, 0.0)),
                ("orders".into(), (300.0, 0.0)),
            ],
        );
        assert_eq!(nodes[1].outputs[1].name, "int · FK");
        assert_eq!(edges.len(), 1);
        assert_eq!(
            (
                edges[0].from.0.as_ref(),
                edges[0].to.0.as_ref(),
                edges[0].to.1.as_ref()
            ),
            ("orders", "users", "id")
        );
    }

    #[test]
    fn a_table_without_a_place_is_left_out() {
        let (nodes, edges) = diagram(&[DbTable::new("orders", [])], &[]);
        assert!(nodes.is_empty() && edges.is_empty());
    }
}
