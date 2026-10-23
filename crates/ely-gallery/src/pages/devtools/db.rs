use ely_gpui_component::{
    canvas::Viewport,
    devtools::{
        Connection, ConnectionForm, ConnectionManager, ConnectionState, DbTable, ERDiagram, Engine,
        Field, Index, IndexManager, Schema, SchemaTree, TableStructureEditor, TestState,
    },
    theme::{ActiveTheme, Radius},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::ui::{change, keep, section};

pub(super) fn tables() -> Vec<DbTable> {
    vec![
        DbTable::new(
            "users",
            [
                Field::new("id", "int").primary(),
                Field::new("email", "text"),
                Field::new("name", "text").nullable(),
                Field::new("created_at", "timestamptz").default_value("now()"),
            ],
        ),
        DbTable::new(
            "orders",
            [
                Field::new("id", "int").primary(),
                Field::new("user_id", "int").refers("users", "id"),
                Field::new("total", "numeric"),
                Field::new("placed_at", "timestamptz"),
            ],
        ),
        DbTable::new(
            "order_items",
            [
                Field::new("order_id", "int").refers("orders", "id"),
                Field::new("sku", "text"),
                Field::new("quantity", "int").default_value("1"),
            ],
        ),
        DbTable::new(
            "recent_orders",
            [Field::new("id", "int"), Field::new("total", "numeric")],
        )
        .view(),
    ]
}

/// The connections demo's list, the one picked, the one in the form, and its last test.
struct Desk {
    connections: Vec<Connection>,
    selected: Option<SharedString>,
    editing: Connection,
    test: Option<TestState>,
}

impl Desk {
    fn new() -> Self {
        let mut shop = Connection::new("shop", "Shop", Engine::Postgres);
        shop.database = "shop".into();
        shop.user = "app".into();
        shop.state = ConnectionState::Connected;
        let mut cache = Connection::new("cache", "Cache", Engine::Redis);
        cache.host = "10.0.0.12".into();
        let mut analytics = Connection::new("analytics", "Analytics", Engine::MySql);
        analytics.host = "db.internal".into();
        analytics.database = "events".into();
        analytics.state = ConnectionState::Failed("Password rejected for user report".into());
        let mut notes = Connection::new("notes", "Notes", Engine::Sqlite);
        notes.database = "~/notes.db".into();
        Self {
            editing: shop.clone(),
            connections: vec![shop, cache, analytics, notes],
            selected: Some("shop".into()),
            test: None,
        }
    }
}

pub fn connections(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let desk = keep("devtools-connections", Desk::new, window, cx);
    let now = desk.read(cx);
    let (connections, selected, editing, test) = (
        now.connections.clone(),
        now.selected.clone(),
        now.editing.clone(),
        now.test.clone(),
    );
    let [picked, opened, edited, removed, made, tested, saved] = [(); 7].map(|_| desk.clone());
    let mut form = ConnectionForm::new("devtools-connection-form", editing)
        .on_test(move |connection, _, cx| {
            change(&tested, cx, |desk| {
                desk.test = Some(match connection.engine {
                    Engine::MySql => TestState::Failed("Password rejected".into()),
                    _ => TestState::Passed(23),
                })
            })
        })
        .on_save(move |connection, _, cx| {
            change(&saved, cx, |desk| {
                match desk
                    .connections
                    .iter_mut()
                    .find(|each| each.key == connection.key)
                {
                    Some(each) => *each = connection.clone(),
                    None => desk.connections.push(connection.clone()),
                }
                desk.selected = Some(connection.key.clone());
                desk.test = None;
            })
        });
    if let Some(test) = test {
        form = form.test(test);
    }
    section(
        "ConnectionManager · ConnectionForm",
        "Saved connections with their engine and address, or why one failed. Enter or a double press connects; the pencil opens one in the form, where Test tries it and Save keeps it.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(
                div().w(px(340.)).child(
                    ConnectionManager::new("devtools-connections", connections)
                        .selected(selected)
                        .on_select(move |key, _, cx| change(&picked, cx, |desk| desk.selected = Some(key.clone())))
                        .on_open(move |key, _, cx| {
                            change(&opened, cx, |desk| {
                                let each = desk.connections.iter_mut().find(|each| each.key == *key).expect("a listed connection");
                                each.state = ConnectionState::Connected;
                            })
                        })
                        .on_edit(move |key, _, cx| {
                            change(&edited, cx, |desk| {
                                desk.editing = desk.connections.iter().find(|each| each.key == *key).expect("a listed connection").clone();
                                desk.test = None;
                            })
                        })
                        .on_remove(move |key, _, cx| change(&removed, cx, |desk| desk.connections.retain(|each| each.key != *key)))
                        .on_new(move |_, cx| {
                            change(&made, cx, |desk| {
                                let key = SharedString::from(format!("new-{}", desk.connections.len() + 1));
                                desk.editing = Connection::new(key, "", Engine::Postgres);
                                desk.test = None;
                            })
                        }),
                ),
            )
            .child(div().w(px(360.)).child(form)),
    )
}

/// The schema demo's tree selection and where each table sits in the diagram.
struct Map {
    selected: Vec<SharedString>,
    places: Vec<(SharedString, (f32, f32))>,
    view: Viewport,
}

pub fn schema(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let map = keep(
        "devtools-schema",
        || Map {
            selected: vec!["public.orders".into()],
            places: vec![
                ("order_items".into(), (20.0, 40.0)),
                ("orders".into(), (300.0, 20.0)),
                ("users".into(), (580.0, 60.0)),
                ("recent_orders".into(), (20.0, 230.0)),
            ],
            view: Viewport::new(0.0, 0.0, 0.7),
        },
        window,
        cx,
    );
    let now = map.read(cx);
    let (selected, places, view) = (now.selected.clone(), now.places.clone(), now.view);
    let [picked, moved, viewed] = [(); 3].map(|_| map.clone());
    let theme = cx.theme();
    let boxed = |width: f32, height: f32| {
        div()
            .w(px(width))
            .h(px(height))
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .overflow_hidden()
    };
    section(
        "SchemaTree · ERDiagram",
        "A database's schemas, tables and views, each field with its type and keys marked. The diagram draws each table as a box of fields and a line from every foreign key to the field it points at; a table's title drags it.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(
                boxed(260.0, 320.0).p_1().child(
                    SchemaTree::new(
                        "devtools-schema-tree",
                        [Schema { name: "public".into(), tables: tables() }],
                    )
                    .open(["public", "public.orders"])
                    .selected(selected)
                    .on_select(move |keys, _, cx| change(&picked, cx, |map| map.selected = keys.to_vec())),
                ),
            )
            .child(
                boxed(560.0, 320.0).child(
                    ERDiagram::new("devtools-erd", view, tables(), places)
                        .on_move(move |name, (dx, dy), _, cx| {
                            change(&moved, cx, |map| {
                                let place = map.places.iter_mut().find(|(each, _)| each == name).expect("a placed table");
                                place.1 = (place.1.0 + dx, place.1.1 + dy);
                            })
                        })
                        .on_viewport(move |next, _, cx| change(&viewed, cx, |map| map.view = next)),
                ),
            ),
    )
}

/// The structure demo's fields and indexes.
struct Build {
    fields: Vec<Field>,
    indexes: Vec<Index>,
}

pub fn structure(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let build = keep(
        "devtools-structure",
        || Build {
            fields: tables()[1].fields.clone(),
            indexes: vec![
                Index {
                    name: "orders_pkey".into(),
                    fields: vec!["id".into()],
                    unique: true,
                    method: "btree".into(),
                },
                Index {
                    name: "orders_user_placed".into(),
                    fields: vec!["user_id".into(), "placed_at".into()],
                    unique: false,
                    method: "btree".into(),
                },
            ],
        },
        window,
        cx,
    );
    let now = build.read(cx);
    let (fields, indexes) = (now.fields.clone(), now.indexes.clone());
    let names: Vec<SharedString> = fields.iter().map(|field| field.name.clone()).collect();
    let [edited, dropped, created] = [(); 3].map(|_| build.clone());
    section(
        "TableStructureEditor · IndexManager",
        "A table's fields edited in place: a name or a default rewritten on a press, a type, whether it may be empty, and the key. Its indexes list their fields and method, and a new one is drawn up from the table's fields.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(
                div().w(px(660.)).child(TableStructureEditor::new("devtools-structure", fields, move |next, _, cx| change(&edited, cx, |build| build.fields = next)),
                ),
            )
            .child(
                div().w(px(300.)).child(
                    IndexManager::new("devtools-indexes", indexes, names, move |index, _, cx| change(&created, cx, |build| build.indexes.push(index)), move |name, _, cx| change(&dropped, cx, |build| build.indexes.retain(|index| index.name != *name))),
                ),
            ),
    )
}
