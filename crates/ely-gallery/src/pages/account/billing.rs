use ely_gpui_component::account::{
    BillingHistory, Invitation, InvitationList, Invoice, InvoiceState, Member, PaymentMethodForm,
    Quota, Role, Standing, Subscription, SubscriptionCard, TeamMemberTable, UpgradePrompt,
    UsageQuota,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::ui::{change, keep, section};

/// The plan demo: whether it was canceled, and whether a card was saved.
#[derive(Default)]
struct Plan {
    canceled: bool,
    card: Option<SharedString>,
}

pub fn plan(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("account-plan", Plan::default, window, cx);
    let now = state.read(cx);
    let (canceled, card) = (now.canceled, now.card.clone());
    let [cancel, resume, saved] = [(); 3].map(|_| state.clone());
    let standing = match canceled {
        true => Standing::Canceled {
            ends: "1 October 2026".into(),
        },
        false => Standing::Active {
            renews: "1 October 2026".into(),
        },
    };
    let quota = |key: &str, name: &str, used: f64, limit: f64, unit: &str| Quota {
        key: key.to_string().into(),
        name: name.to_string().into(),
        used,
        limit,
        unit: unit.to_string().into(),
    };
    section(
        "SubscriptionCard / PlanBadge · UsageQuota · PaymentMethodForm",
        "The plan with where it stands and what comes next; Cancel asks twice and Keep plan takes it back. Each limit's bar turns amber near it and red at it. A card saves once its number checks out, its month is ahead and its code is whole.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div().w(px(340.)).flex().flex_col().gap_6().child(
                    SubscriptionCard::new(
                        "account-plan",
                        Subscription { plan: "Pro".into(), price: 20.0, currency: "USD".into(), yearly: false, standing },
                    )
                    .on_change(|_, _| log::info!("gallery: change plan"))
                    .on_cancel(move |_, cx| change(&cancel, cx, |plan| plan.canceled = true))
                    .on_resume(move |_, cx| change(&resume, cx, |plan| plan.canceled = false)),
                )
                .child(
                    UsageQuota::new(
                        "account-quota",
                        [
                            quota("seats", "Seats across every workspace you own", 9.0, 12.0, "seats"),
                            quota("storage", "Storage", 96.5, 100.0, "GB"),
                            quota("runs", "Automation runs", 1_240.0, 2_000.0, "runs"),
                        ],
                    )
                    .resets("1 October"),
                ),
            )
            .child(div().w(px(360.)).flex().flex_col().gap_3().child(
                PaymentMethodForm::new("account-card-form", move |card, _, cx| {
                    let saved_as: SharedString = format!("{} ending {}", card.brand.name(), &card.number[card.number.len() - 4..]).into();
                    change(&saved, cx, move |plan| plan.card = Some(saved_as))
                }),
            )
            .children(card.map(|card| div().text_sm().child(format!("Saved: {card}"))))),
    )
}

pub fn history(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let invoice = |key: &str, date: &str, description: &str, amount: f64, state| Invoice {
        key: key.to_string().into(),
        date: date.to_string().into(),
        description: description.to_string().into(),
        amount,
        currency: "USD".into(),
        state,
    };
    section(
        "BillingHistory · UpgradePrompt / Paywall",
        "Past bills with what each was for, its amount, date and state, and a download. A prompt past a limit stands as one line, or as a paywall with what the next plan brings.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(div().w(px(400.)).child(
                BillingHistory::new(
                    "account-bills",
                    [
                        invoice("sep", "1 Sep 2026", "Pro, monthly", 20.0, InvoiceState::Due),
                        invoice("aug", "1 Aug 2026", "Pro, monthly", 20.0, InvoiceState::Paid),
                        invoice("jul", "1 Jul 2026", "Pro, monthly", 20.0, InvoiceState::Failed),
                        invoice("jun", "12 Jun 2026", "Extra seats", -15.0, InvoiceState::Refunded),
                    ],
                )
                .on_download(|key, _, _| log::info!("gallery: download {key}")),
            ))
            .child(
                div()
                    .w(px(360.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        UpgradePrompt::new("account-nudge", "9 of 10 projects used", |_, _| log::info!("gallery: upgrade"))
                            .body("Pro has no limit on projects.")
                            .on_dismiss(|_, _| log::info!("gallery: not now")),
                    )
                    .child(
                        UpgradePrompt::new("account-wall", "Version history is on Pro", |_, _| log::info!("gallery: upgrade"))
                            .body("See and restore every version of a document.")
                            .benefit("Every version, kept a year")
                            .benefit("Restore with one press")
                            .benefit("Compare any two")
                            .on_dismiss(|_, _| log::info!("gallery: not now")),
                    ),
            ),
    )
}

fn roles() -> Vec<Role> {
    let role = |key: &str, name: &str, description: &str| Role {
        key: key.to_string().into(),
        name: name.to_string().into(),
        description: description.to_string().into(),
    };
    vec![
        role("member", "Member", "Projects and documents"),
        role("admin", "Admin", "Everything, billing too"),
        role("guest", "Guest", "Only what is shared"),
    ]
}

/// The team demo: members and invitations out.
struct Team {
    members: Vec<Member>,
    invitations: Vec<Invitation>,
    sent: usize,
}

fn team() -> Team {
    let member = |key: &str, name: &str, role: &str, seen: &str, owner| Member {
        key: key.to_string().into(),
        name: name.to_string().into(),
        email: format!("{key}@atlas.dev").into(),
        role: role.to_string().into(),
        seen: seen.to_string().into(),
        image: None,
        owner,
    };
    Team {
        members: vec![
            member("ada", "Ada Lovelace", "admin", "Active now", true),
            member("grace", "Grace Hopper", "admin", "Active today", false),
            member("alan", "Alan Turing", "member", "3 days ago", false),
        ],
        invitations: vec![
            Invitation {
                key: "kat".into(),
                email: "katherine@nasa.gov".into(),
                role: "member".into(),
                sent: "Sent 2 days ago".into(),
                expired: false,
            },
            Invitation {
                key: "edsger".into(),
                email: "edsger@tue.nl".into(),
                role: "guest".into(),
                sent: "Sent 9 days ago".into(),
                expired: true,
            },
        ],
        sent: 0,
    }
}

pub fn members(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("account-team", team, window, cx);
    let now = state.read(cx);
    let (members, invitations) = (now.members.clone(), now.invitations.clone());
    let [role, removed, invited, revoked] = [(); 4].map(|_| state.clone());
    section(
        "TeamMemberTable · RoleSelector · InvitationList",
        "Members with a role to change, each role saying what it may do, and a Remove that asks twice; the owner stays. Invitations out resend or revoke, and an address with a role invites someone new.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_6()
            .w(px(640.))
            .child(
                TeamMemberTable::new("account-members", members, roles(), move |key, next, _, cx| {
                        let (key, next) = (key.clone(), next.clone());
                        change(&role, cx, move |team| team.members.iter_mut().filter(|member| member.key == key).for_each(|member| member.role = next.clone()))
                    }, move |key, _, cx| {
                        let key = key.clone();
                        change(&removed, cx, move |team| team.members.retain(|member| member.key != key))
                    }),
            )
            .child(
                InvitationList::new("account-invites", invitations, roles(), move |email, role, _, cx| {
                        let (email, role) = (email.clone(), role.clone());
                        change(&invited, cx, move |team| {
                            team.sent += 1;
                            team.invitations.insert(0, Invitation { key: format!("new-{}", team.sent).into(), email, role, sent: "Sent just now".into(), expired: false });
                        })
                    }, |key, _, _| log::info!("gallery: resend {key}"), move |key, _, cx| {
                        let key = key.clone();
                        change(&revoked, cx, move |team| team.invitations.retain(|each| each.key != key))
                    }),
            ),
    )
}
