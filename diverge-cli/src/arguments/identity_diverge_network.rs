//! What `diverge identity.diverge.network` takes.

use clap::Subcommand;

/// `identity.diverge.network`, Diverge's own OpenID Connect provider,
/// worked on from this host. Every command but `accounts list` and
/// `accounts login` names an account by its `<ISSUER>` and its `<ID>`,
/// as `accounts list` prints them. An account's issuer is who issued
/// its own sign-in — `github`, `google` — and never
/// `identity.diverge.network` itself.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IdentityDivergeNetwork {
    /// The host's accounts there: each a user of an issuer — GitHub,
    /// Google — which sets a username once, and owns every subject
    /// that begins with it.
    #[command(subcommand)]
    Accounts(Accounts),
    /// An account's subjects.
    #[command(subcommand)]
    Subjects(Subjects),
    /// A subject's refresh tokens: where it is signed in.
    #[command(name = "refresh-tokens", subcommand)]
    RefreshTokens(RefreshTokens),
}

/// The host's accounts at `identity.diverge.network`, under the root.
/// No command prints a token.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Accounts {
    /// Every account on the host, as `{"account","issuer","id"}`, one
    /// JSON line each. Nothing is asked of anybody.
    List,
    /// What an account's issuer says of its user, now: one JSON line,
    /// `{"account","issuer","id","info"}`, the `info` the issuer's
    /// own object — a GitHub user, or Google's OpenID Connect claims
    /// — as the issuer wrote it. Asked of the issuer every time, and
    /// kept nowhere.
    Info {
        /// The account's issuer, by its name.
        issuer: String,
        /// The account's id at that issuer.
        id: String,
    },
    /// An account's username: the first part of every subject it
    /// owns.
    #[command(subcommand)]
    Username(Username),
    /// Sign in to an account, at its issuer, by a device code: the
    /// code printed, the issuer's page opened in this host's browser,
    /// and the command ended there. The wait for the code's approval
    /// goes on apart from it, and the account is written once the
    /// issuer grants it — replacing one already there — which
    /// `accounts list` then shows.
    Login {
        /// The issuer to sign in at, by its name: `github`, `google`.
        /// A name that is none is refused with the ones there are.
        issuer: String,
        /// Open no browser: the line printed names the page, for a
        /// person to open.
        #[arg(long)]
        no_browser: bool,
    },
}

/// An account's username at `identity.diverge.network`: what its
/// subjects, `<USERNAME>.<name>`, begin with. An account has none
/// until it sets one, and can sign no subject in until it has.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Username {
    /// The account's username, one JSON line: `{"account","username"}`,
    /// the `username` null where it has set none.
    Get {
        /// The account's issuer, by its name.
        issuer: String,
        /// The account's id at that issuer.
        id: String,
    },
    /// Set the account's username, once: it is never changed and never
    /// given up afterwards. One JSON line, `{"account","username"}`.
    /// Refused where the account has one already, and where the
    /// username is another account's.
    Set {
        /// The account's issuer, by its name.
        issuer: String,
        /// The account's id at that issuer.
        id: String,
        /// The username: 1 to 39 of `a-z`, `0-9`, `-` and `_`, which
        /// begins and ends with a letter or a digit, and has no two of
        /// `-` and `_` together.
        username: String,
    },
}

/// An account's subjects at `identity.diverge.network`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Subjects {
    /// Every subject of the account's that is signed in somewhere, as
    /// `{"subject","name","refresh_tokens"}`, one JSON line each.
    List {
        /// The account's issuer.
        issuer: String,
        /// The account's id.
        id: String,
    },
}

/// A subject's refresh tokens at `identity.diverge.network`: one for
/// each place it is signed in. No command prints one.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum RefreshTokens {
    /// Every refresh token of the account's subject of that name, the
    /// oldest first, as
    /// `{"id","created","ip","user_agent","last_used","last_ip"}`,
    /// one JSON line each.
    List {
        /// The account's issuer.
        issuer: String,
        /// The account's id.
        id: String,
        /// The subject's name: its third part.
        name: String,
    },
    /// End one of them: what it signed in is signed out once the
    /// tokens it holds expire. Printed as `{"ended":…}`.
    End {
        /// The account's issuer.
        issuer: String,
        /// The account's id.
        id: String,
        /// The subject's name: its third part.
        name: String,
        /// The refresh token's id, as `refresh-tokens list` prints it.
        refresh_token: String,
    },
}
