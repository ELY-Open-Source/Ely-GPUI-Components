use ely_gpui_component::{
    account::{
        Account, AccountSwitcher, ApiKey, ApiKeyManager, Profile, ProfileCard, ProfileEditor,
        Session, SessionList, UserMenu, Workspace, WorkspaceSwitcher,
    },
    buttons::Button,
    data_display::Presence,
    menus::MenuItem,
    primitives::IconName,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

/// The switchers' demo: the account and workspace in use.
struct Places {
    account: SharedString,
    workspace: SharedString,
}

pub fn switchers(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep(
        "account-places",
        || Places {
            account: "work".into(),
            workspace: "atlas".into(),
        },
        window,
        cx,
    );
    let now = state.read(cx);
    let (account, workspace) = (now.account.clone(), now.workspace.clone());
    let [to_account, to_workspace] = [(); 2].map(|_| state.clone());
    let accounts = [
        Account {
            key: "work".into(),
            name: "Ada Lovelace".into(),
            email: "ada@atlas.dev".into(),
            image: None,
        },
        Account {
            key: "home".into(),
            name: "Ada Lovelace".into(),
            email: "ada@hey.com".into(),
            image: None,
        },
    ];
    let workspaces = [
        Workspace {
            key: "atlas".into(),
            name: "Atlas".into(),
            detail: "Pro · 12 members".into(),
        },
        Workspace {
            key: "field".into(),
            name: "Field Notes".into(),
            detail: "Free · 3 members".into(),
        },
    ];
    section(
        "AccountSwitcher · WorkspaceSwitcher / OrgSwitcher · UserMenu",
        "The account and the workspace in use, each a row that opens a menu of the others; the avatar opens who is signed in, the owner's rows, and Sign out.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div().w(px(260.)).flex().flex_col().gap_2().child(
                    WorkspaceSwitcher::new("account-workspaces", workspaces, workspace, move |key, _, cx| change(&to_workspace, cx, |places| places.workspace = key.clone()))
                        .on_create(|_, _| log::info!("gallery: create workspace")),
                )
                .child(
                    AccountSwitcher::new("account-accounts", accounts, account, move |key, _, cx| change(&to_account, cx, |places| places.account = key.clone()))
                        .on_add(|_, _| log::info!("gallery: add account")),
                ),
            )
            .child(probe(
                "account-user",
                UserMenu::new("account-user", "Ada Lovelace", "ada@atlas.dev", |_, _| log::info!("gallery: sign out"))
                    .presence(Presence::Online)
                    .item(MenuItem::new("Profile").icon(IconName::User).on_click(|_, _| log::info!("gallery: profile")))
                    .item(MenuItem::new("Settings").icon(IconName::Settings).on_click(|_, _| log::info!("gallery: settings"))),
            )),
    )
}

pub fn profile(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep(
        "account-profile",
        || Profile {
            name: "Ada Lovelace".into(),
            username: "ada".into(),
            email: "ada@atlas.dev".into(),
            bio: "Writes the notes that make machines do more than count.".into(),
            location: "London".into(),
            picture: None,
        },
        window,
        cx,
    );
    let now = state.read(cx).clone();
    let saved = state.clone();
    section(
        "ProfileCard · ProfileEditor",
        "Someone at a glance, and their profile to edit: Save waits for a change, a name and an email that reads; Discard puts the saved profile back.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div().w(px(320.)).child(
                    ProfileCard::new("account-card", now.name.clone())
                        .title("Analyst, Atlas")
                        .presence(Presence::Online)
                        .bio(now.bio.clone())
                        .detail(IconName::Mail, now.email.clone())
                        .detail(IconName::MapPin, now.location.clone())
                        .action(Button::new("account-card-message", "Message").primary())
                        .action(Button::new("account-card-profile", "View profile")),
                ),
            )
            .child(div().w(px(400.)).child(ProfileEditor::new("account-editor", now, move |profile, _, cx| {
                let profile = profile.clone();
                change(&saved, cx, move |now| *now = profile)
            }))),
    )
}

/// The security demo: sessions still signed in, keys, and a secret just made.
struct Security {
    sessions: Vec<Session>,
    keys: Vec<ApiKey>,
    secret: Option<SharedString>,
    made: usize,
}

fn security() -> Security {
    let session =
        |key: &str, icon, device: &str, client: &str, place: &str, seen: &str, current| Session {
            key: key.to_string().into(),
            icon,
            device: device.to_string().into(),
            client: client.to_string().into(),
            place: place.to_string().into(),
            seen: seen.to_string().into(),
            current,
        };
    Security {
        sessions: vec![
            session(
                "phone",
                IconName::Smartphone,
                "iPhone 17",
                "Ely for iOS",
                "Lisbon, Portugal",
                "2 hours ago",
                false,
            ),
            session(
                "mac",
                IconName::Monitor,
                "MacBook Pro",
                "Ely 4.2",
                "Lisbon, Portugal",
                "Active now",
                true,
            ),
            session(
                "office",
                IconName::Monitor,
                "Office desktop",
                "Firefox 150",
                "Porto, Portugal",
                "3 days ago",
                false,
            ),
        ],
        keys: vec![
            ApiKey {
                key: "ci".into(),
                name: "CI".into(),
                hint: "ely_live_…4f2a".into(),
                created: "Made 3 Sep 2026".into(),
                used: "Last used today".into(),
            },
            ApiKey {
                key: "backup".into(),
                name: "Backups".into(),
                hint: "ely_live_…91c0".into(),
                created: "Made 12 Aug 2026".into(),
                used: "Never used".into(),
            },
        ],
        secret: None,
        made: 0,
    }
}

pub fn security_page(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("account-security", security, window, cx);
    let now = state.read(cx);
    let (sessions, keys, secret) = (now.sessions.clone(), now.keys.clone(), now.secret.clone());
    let [out, others, made, revoked, done] = [(); 5].map(|_| state.clone());
    let mut manager = ApiKeyManager::new(
        "account-keys",
        keys,
        move |name, _, cx| {
            let name = name.clone();
            change(&made, cx, move |security| {
                security.made += 1;
                let key: SharedString = format!("new-{}", security.made).into();
                security.keys.insert(
                    0,
                    ApiKey {
                        key,
                        name,
                        hint: "ely_live_…7d3e".into(),
                        created: "Made today".into(),
                        used: "Never used".into(),
                    },
                );
                security.secret = Some("ely_live_5f1c0a9e2b7d4c8a3e6f7d3e".into());
            })
        },
        move |key, _, cx| {
            let key = key.clone();
            change(&revoked, cx, move |security| {
                security.keys.retain(|each| each.key != key)
            })
        },
        move |_, cx| change(&done, cx, |security| security.secret = None),
    );
    if let Some(secret) = secret {
        manager = manager.secret(secret);
    }
    section(
        "SessionList · ApiKeyManager",
        "Where the account is signed in, this device first, each other one signing out on its own or all at once after a second press. API keys show their ends only; a new key's secret shows once.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_8()
            .child(div().w(px(380.)).child(
                SessionList::new("account-sessions", sessions, move |key, _, cx| {
                        let key = key.clone();
                        change(&out, cx, move |security| security.sessions.retain(|each| each.key != key))
                    })
                    .on_sign_out_others(move |_, cx| change(&others, cx, |security| security.sessions.retain(|each| each.current))),
            ))
            .child(div().w(px(420.)).child(manager)),
    )
}
