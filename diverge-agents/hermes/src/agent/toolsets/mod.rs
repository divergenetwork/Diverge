//! Toolset definitions — each capability and its arguments.
//!
//! AUTH IS AN ARGUMENT here exactly as it is for
//! [providers](super::provider): a toolset's credential is a member
//! of its own structure, beside the facts that are not credentials —
//! an endpoint, a backend's name. A static credential (an api key, a
//! token) is a string, read once; spotify's login refreshes, and is
//! kept as the OAuth providers' are. Toolsets with nothing to
//! configure are `Option<bool>`s on [`Toolsets`] (absent is off) and
//! get no file. A toolset whose only argument is a key is a
//! structure of that one member, in a file of its own. Nothing about
//! a tool's auth rides a mount, and the request carries no
//! environment for it to ride.
//!
//! Where two places are applied under one variable (both generation
//! tools under `FAL_KEY`; x_search and the xai provider under
//! `XAI_API_KEY`), an agent that names both names ONE key: the same
//! value twice is applied once, and two different values are the
//! registration's refusal, naming the variable and never a value.
//!
//! DROPPED from the previous vocabulary:
//! - `context_engine` — ships zero tools in stock Hermes; the
//!   toolset is a socket for an engine plugin the image does not
//!   carry, so there is nothing to enable.
//! - `yuanbao` — its tools call a live Yuanbao platform adapter
//!   that an api_server-only gateway never creates.
//! - `delegation` — sub-agents. On the API server every delegation
//!   is BACKGROUND: the run ends before its children do, and their
//!   results wake the session as a turn nobody drives, off every
//!   stream a run has. Observable only through end-of-turn hooks,
//!   which is a different product than a stream — so the harness
//!   turns the toolset OFF, and it is not a switch a request can
//!   throw.
//!
//! Still absent for their unchanged non-auth reasons: skills
//! (mounts decide it — directories under
//! `/root/.hermes/external-skills/`, each holding a `SKILL.md`, which
//! the harness names to Hermes read-only), `computer_use` (a display
//! server is not an
//! argument), `stt` (not a model toolset at all), `clarify` (needs
//! wiring this wire does not have), `discord`/`discord_admin`
//! (hard-restricted to Hermes's Discord platform), `cronjob` (an
//! ephemeral container has no future to schedule into). `spotify`
//! is not on this list: its login refreshes, and the program keeps
//! the refreshed login. Hermes's managed Nous
//! tool gateway (a refreshing credential several toolsets can route
//! through) stays unsupported with the Nous provider.

pub mod browser;
pub mod homeassistant;
pub mod image_gen;
pub mod spotify;
pub mod tts;
pub mod video_gen;
pub mod web;
pub mod x_search;

mod toolsets;

pub use toolsets::*;
