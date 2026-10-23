use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    collab::{AccessList, InviteInput, LinkAccess, Member, Role, ShareDialog},
    data_display::{Activity, ActivityFeed},
    forms::is_email,
    primitives::IconName,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, ToSpan};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

fn member(name: &str, email: &str, role: Role) -> Member {
    Member {
        name: name.to_string().into(),
        email: email.to_string().into(),
        picture: None,
        role,
        owner: false,
    }
}

fn members() -> Vec<Member> {
    vec![
        Member {
            owner: true,
            ..member("You", "you@example.com", Role::Edit)
        },
        member("Ada Park", "ada@example.com", Role::Edit),
        member("Grace Lin", "grace@example.com", Role::Comment),
        member("Alan Reyes", "alan@example.com", Role::View),
    ]
}

pub fn sharing(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = keep("collab-share-open", || false, window, cx);
    let emails = keep(
        "collab-emails",
        || {
            vec![
                SharedString::from("mae@example.com"),
                SharedString::from("tim@example"),
            ]
        },
        window,
        cx,
    );
    let role = keep("collab-invite-role", || Role::Edit, window, cx);
    let people = keep("collab-members", members, window, cx);
    let link = keep("collab-link", || None::<Role>, window, cx);
    let dialog = open.read(cx).then(|| {
        let (now_emails, now_role, now_people, now_link) = (
            emails.read(cx).clone(),
            *role.read(cx),
            people.read(cx).clone(),
            *link.read(cx),
        );
        let (close, typed, invited, cleared, picked) = (
            open.clone(),
            emails.clone(),
            people.clone(),
            emails.clone(),
            role.clone(),
        );
        let (roled, removed) = (people.clone(), people.clone());
        ShareDialog::new("collab-share", "Lift", move |_, cx| set(&close, false, cx))
            .invite(
                InviteInput::new(
                    "collab-invite",
                    now_emails,
                    move |next, _, cx| set(&typed, next.to_vec(), cx),
                    move |_, cx| {
                        let (added, role) = (cleared.read(cx).clone(), *picked.read(cx));
                        invited.update(cx, |people, cx| {
                            people.extend(added.iter().filter(|email| is_email(email)).map(
                                |email| {
                                    let (name, _) =
                                        email.split_once('@').expect("an address holds @");
                                    member(name, email, role)
                                },
                            ));
                            cx.notify();
                        });
                        set(&cleared, Vec::new(), cx)
                    },
                )
                .role(now_role, move |next, _, cx| set(&role, next, cx)),
            )
            .access(
                AccessList::new("collab-access", now_people)
                    .on_role(move |ix, next, _, cx| {
                        roled.update(cx, |people, cx| {
                            people[ix].role = next;
                            cx.notify();
                        })
                    })
                    .on_remove(move |ix, _, cx| {
                        removed.update(cx, |people, cx| {
                            people.remove(ix);
                            cx.notify();
                        })
                    }),
            )
            .link(LinkAccess::new(
                "collab-link",
                "https://example.com/d/lift",
                now_link,
                move |next, _, cx| set(&link, next, cx),
            ))
    });
    section(
        "ShareDialog / InviteInput / PermissionSelect / AccessList",
        "Sharing in one place: add people by email, each chip marked until it is an address; choose what they may do; change or take away anyone's access; and choose who the link lets in.",
        cx,
    )
    .child(probe(
        "collab-share",
        div().flex().child(
            Button::new("collab-share-open", "Share")
                .variant(ButtonVariant::Primary)
                .icon(IconName::UserPlus)
                .on_click(move |_, _, cx| set(&open, true, cx)),
        ),
    ))
    .children(dialog)
}

pub fn activity(cx: &mut App) -> impl IntoElement + use<> {
    let now = Timestamp::now();
    let entry = |key: &str, actor: &str, verb: &str, object: &str, minutes: i64| {
        Activity::new(
            key.to_string(),
            actor.to_string(),
            verb.to_string(),
            now - minutes.minutes(),
        )
        .object(object.to_string())
    };
    section(
        "ActivityLog → data_display::ActivityFeed",
        "What people did in the document, newest first.",
        cx,
    )
    .child(
        div().w(px(560.)).child(
            ActivityFeed::new("collab-activity")
                .entry(entry(
                    "reply",
                    "Alan Reyes",
                    "replied on",
                    "blends in gamma space",
                    4,
                ))
                .entry(entry(
                    "suggest",
                    "Grace Lin",
                    "suggested a change to",
                    "Lift",
                    35,
                ))
                .entry(entry("resolve", "Ada Park", "resolved", "On Lift", 180))
                .entry(entry("share", "You", "shared", "Lift", 1440)),
        ),
    )
}
