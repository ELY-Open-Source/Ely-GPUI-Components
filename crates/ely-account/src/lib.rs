//! Account, chapter 36: sign-in and sign-up, profiles, sessions, keys, teams, switchers, plans and payment.
#![deny(missing_docs)]

use ely_core::{
    buttons, data_display, feedback, forms, i18n, layout, menus, primitives, theme, typography,
};

mod billing;
mod factor;
mod keys;
mod login;
mod mail;
mod oauth;
mod payment;
mod profile;
mod sessions;
mod signup;
mod switcher;
mod team;
mod upgrade;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use billing::{
    BillingHistory, Invoice, InvoiceState, Quota, Standing, Subscription, SubscriptionCard,
    UsageQuota, quota_line,
};
pub use factor::TwoFactorInput;
pub use keys::{ApiKey, ApiKeyManager};
pub use login::{Login, LoginForm};
pub use mail::{ForgotPassword, MagicLinkForm};
pub use oauth::OAuthButtons;
pub use payment::{CardBrand, CardDetails, ExpiryError, PaymentMethodForm, expiry, luhn};
pub use profile::{Profile, ProfileCard, ProfileEditor};
pub use sessions::{Session, SessionList};
pub use signup::{Signup, SignupForm, password_rules};
pub use switcher::{Account, AccountSwitcher, UserMenu, Workspace, WorkspaceSwitcher};
pub use team::{Invitation, InvitationList, Member, Role, RoleSelector, TeamMemberTable};
pub use upgrade::UpgradePrompt;
