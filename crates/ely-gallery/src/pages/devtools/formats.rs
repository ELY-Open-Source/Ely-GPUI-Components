use ely_gpui_component::{
    devtools::{JsonEditor, JsonViewer, XmlViewer, YamlViewer},
    editor::CodeEditor,
    forms::TextInput,
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::ui::section;

const ORDER: &str = r#"{
  "_id": "66f1c2a9e4b0",
  "customer": { "name": "Ada Lovelace", "email": "ada@example.com" },
  "items": [
    { "sku": "LAMP-01", "quantity": 1, "price": 89.0 },
    { "sku": "BULB-E27", "quantity": 4, "price": 6.5 }
  ],
  "paid": true,
  "shipped_at": null,
  "tags": ["gift", "express"]
}"#;

const DEPLOYMENT: &str = "apiVersion: apps/v1
kind: Deployment
metadata:
  name: web
  labels:
    app: web
spec:
  replicas: 3
  template:
    spec:
      containers:
        - name: web
          image: registry.example.com/web:1.8.2
          ports:
            - containerPort: 8080
";

const FEED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Release notes</title>
    <item id="1">
      <title>Faster queries</title>
      <pubDate>Fri, 25 Sep 2026 09:00:00 GMT</pubDate>
    </item>
    <item id="2">
      <title>Dark mode</title>
    </item>
  </channel>
</rss>"#;

pub fn documents(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let search = window.use_keyed_state("devtools-json-search", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Find a key or a value")
    });
    let editor = window.use_keyed_state("devtools-json-editor", cx, |window, cx| {
        CodeEditor::new(
            r#"{"name": "web", "replicas": 3, "ports": [8080, 8443]}"#,
            window,
            cx,
        )
        .language("JSON")
    });
    let yaml_search = window.use_keyed_state("devtools-yaml-search", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Find a key or a value")
    });
    section(
        "DocumentViewer (MongoDB / JSON) → devtools::JsonViewer · JsonViewer / JsonTree · JsonEditor · YamlViewer · XmlViewer",
        "A document as a tree: each key with its value, lists and maps with their counts; a search keeps what holds it and opens the way, and Enter on a value copies its path. The editor writes JSON with Format and Minify and says whether it reads. YAML reads as the same tree; XML as elements with their attributes and text.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(div().w(px(380.)).child(JsonViewer::new("devtools-json", ORDER, &search)))
            .child(div().w(px(420.)).child(JsonEditor::new("devtools-json-editor", &editor)))
            .child(div().w(px(380.)).child(YamlViewer::new("devtools-yaml", DEPLOYMENT, &yaml_search)))
            .child(div().w(px(420.)).child(XmlViewer::new("devtools-xml", FEED))),
    )
}
