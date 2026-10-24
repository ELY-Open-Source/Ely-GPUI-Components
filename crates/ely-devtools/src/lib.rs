//! Developer tools, chapter 33: databases, requests, sockets, containers, data viewers, encoders and a contrast audit.
#![deny(missing_docs)]

use ely_canvas as canvas;
use ely_chat as chat;
use ely_core::{
    buttons, data_display, feedback, forms, i18n, layout, lists, navigation, primitives, tables,
    theme, typography,
};
use ely_editor as editor;

mod audit;
mod collection;
mod connection_form;
mod connections;
mod contrast;
mod datum;
mod document;
mod encode;
mod environment;
mod graphql;
mod json_editor;
mod machines;
mod monitor;
mod queries;
mod redis;
mod regex;
mod request;
mod response;
mod schema;
mod socket;
mod structure;
mod xml;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use audit::{Check, MARK_CONTRAST, TEXT_CONTRAST, audit};
pub use collection::{CollectionTree, Saved, moved};
pub use connection_form::{ConnectionForm, TestState};
pub use connections::{Connection, ConnectionManager, ConnectionState, Engine};
pub use contrast::ColorContrastChecker;
pub use datum::{Datum, Unread, read_json, read_yaml};
pub use document::{JsonViewer, YamlViewer};
pub use encode::{Code, Encoder, base64, hex, unbase64, unhex, unpercent};
pub use environment::{Environment, EnvironmentSelector};
pub use graphql::{GraphQLExplorer, SchemaType, TypeKind};
pub use json_editor::JsonEditor;
pub use machines::{Container, ContainerList, ContainerState, Pod, PodList, PodPhase};
pub use monitor::{Reading, ResourceMonitor};
pub use queries::{Outcome, PlanStep, QueryHistory, QueryPlanViewer, QueryRun};
pub use redis::{RedisKey, RedisKeyBrowser, RedisValue};
pub use regex::{Found, RegexFlag, RegexTester, find};
pub use request::{ApiRequest, ApiRequestBuilder, Auth, Method, percent, resolve};
pub use response::{ApiResponse, HeadersTable, ResponseViewer, reason, status_tone};
pub use schema::{DbTable, ERDiagram, Field, FieldKey, Schema, SchemaTree};
pub use socket::{Direction, SocketMessage, SocketState, WebSocketConsole};
pub use structure::{Index, IndexManager, TableStructureEditor};
pub use xml::{XmlElement, XmlNode, XmlViewer, read_xml};
