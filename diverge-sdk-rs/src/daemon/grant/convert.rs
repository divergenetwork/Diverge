//! A tool's declared grants, as grants of the daemon's.

use super::{Grant, accounts, agents, agents_templates, connected_standalone_tools, daemons, dependency_tools, identities, images, providers, providers_daemons, roles, standalone_tools, standalone_tools_templates, vault, volumes};
use crate::shared::permission::{self, Tags, Within};

/// A tool's grants, as its container declared them at registration and
/// as the daemon judges them: the daemon's grants, each the same
/// actions reaching as the tool's reach says — every one, the kind's
/// own filter naming exactly the records listed, or the filter
/// carrying the tags alone — and, apart from them, since no grant of
/// the daemon's own can say it, the actions over agents that reach
/// `attached`, which the daemon judges by the agents attached to the
/// tool. `never_active` is a daemon grant of its own, its filter
/// `{"active": "never"}`. Nothing is widened: a filter naming records
/// reaches those records and nothing else, a filter of tags alone
/// what the tags test reaches and nothing else, `attached` the tool's
/// attached agents and nothing else, and `never_active` the agents
/// that have never run and nothing else.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Converted {
    /// The grants a role could hold, judged as a role's are.
    pub grants: Vec<Grant>,
    /// The actions over agents held within `attached`, judged against
    /// the agents attached to the tool.
    pub attached_actions: Vec<agents::Over>,
}

impl From<Vec<permission::Grant>> for Converted {
    fn from(grants: Vec<permission::Grant>) -> Self {
        let mut converted = Converted::default();
        for grant in grants {
            match daemon_grant(grant) {
                Ok(grant) => converted.grants.push(grant),
                Err(attached) => converted.attached_actions.extend(attached),
            }
        }
        converted
    }
}

/// The daemon's grant a shared grant is, or — for a grant over agents
/// within `attached`, which no daemon grant can say — the actions it
/// holds, to be judged apart.
fn daemon_grant(grant: permission::Grant) -> Result<Grant, Vec<agents::Over>> {
    Ok(match grant {
        permission::Grant::Agents(permission) => Grant::Agents(match permission {
            permission::agents::Permission::Make(makes) => agents::Permission::Make(makes),
            permission::agents::Permission::Over { actions, within } => {
                let within = match within {
                    permission::agents::Within::Any => Within::Any,
                    permission::agents::Within::Attached => return Err(actions),
                    permission::agents::Within::Agents(agents) => agents_within(permission::Reach::Listed(permission::agents::Listed { agents })),
                    permission::agents::Within::NeverActive => Within::Only(never_active()),
                    permission::agents::Within::Only(tags) => agents_within(permission::Reach::Tags(tags)),
                };
                agents::Permission::Over { actions, within }
            }
            permission::agents::Permission::Tags { actions, within, tags } => agents::Permission::Tags {
                actions,
                within: agents_within(within),
                tags,
            },
        }),
        permission::Grant::AgentsTemplates(permission) => Grant::AgentsTemplates(match permission {
            permission::agents_templates::Permission::Make(makes) => agents_templates::Permission::Make(makes),
            permission::agents_templates::Permission::Over { actions, within } => agents_templates::Permission::Over {
                actions,
                within: agents_templates_within(within),
            },
            permission::agents_templates::Permission::Tags { actions, within, tags } => agents_templates::Permission::Tags {
                actions,
                within: agents_templates_within(within),
                tags,
            },
        }),
        permission::Grant::StandaloneTools(permission) => Grant::StandaloneTools(match permission {
            permission::standalone_tools::Permission::Make(makes) => standalone_tools::Permission::Make(makes),
            permission::standalone_tools::Permission::Over { actions, within } => standalone_tools::Permission::Over {
                actions,
                within: standalone_tools_within(within),
            },
            permission::standalone_tools::Permission::Tags { actions, within, tags } => standalone_tools::Permission::Tags {
                actions,
                within: standalone_tools_within(within),
                tags,
            },
        }),
        permission::Grant::StandaloneToolsTemplates(permission) => Grant::StandaloneToolsTemplates(match permission {
            permission::standalone_tools_templates::Permission::Make(makes) => standalone_tools_templates::Permission::Make(makes),
            permission::standalone_tools_templates::Permission::Over { actions, within } => standalone_tools_templates::Permission::Over {
                actions,
                within: standalone_tools_templates_within(within),
            },
            permission::standalone_tools_templates::Permission::Tags { actions, within, tags } => standalone_tools_templates::Permission::Tags {
                actions,
                within: standalone_tools_templates_within(within),
                tags,
            },
        }),
        permission::Grant::ConnectedStandaloneTools(permission) => Grant::ConnectedStandaloneTools(match permission {
            permission::connected_standalone_tools::Permission::Make(makes) => connected_standalone_tools::Permission::Make(makes),
            permission::connected_standalone_tools::Permission::Over { actions, within } => connected_standalone_tools::Permission::Over {
                actions,
                within: connected_standalone_tools_within(within),
            },
            permission::connected_standalone_tools::Permission::Tags { actions, within, tags } => connected_standalone_tools::Permission::Tags {
                actions,
                within: connected_standalone_tools_within(within),
                tags,
            },
        }),
        permission::Grant::DependencyTools(permission) => Grant::DependencyTools(match permission {
            permission::dependency_tools::Permission::Over { actions, within } => dependency_tools::Permission::Over {
                actions,
                within: dependency_tools_within(within),
            },
            permission::dependency_tools::Permission::Tags { actions, within, tags } => dependency_tools::Permission::Tags {
                actions,
                within: dependency_tools_within(within),
                tags,
            },
        }),
        permission::Grant::Providers(permission) => Grant::Providers(match permission {
            permission::providers::Permission::Make(makes) => providers::Permission::Make(makes),
            permission::providers::Permission::Over { actions, within } => providers::Permission::Over {
                actions,
                within: providers_within(within),
            },
            permission::providers::Permission::Tags { actions, within, tags } => providers::Permission::Tags {
                actions,
                within: providers_within(within),
                tags,
            },
        }),
        permission::Grant::ProvidersDaemons(permission) => Grant::ProvidersDaemons(match permission {
            permission::providers_daemons::Permission::Over { actions, within } => providers_daemons::Permission::Over {
                actions,
                within: providers_daemons_within(within),
            },
        }),
        permission::Grant::Daemons(permission) => Grant::Daemons(match permission {
            permission::daemons::Permission::Make(makes) => daemons::Permission::Make(makes),
            permission::daemons::Permission::Over { actions, within } => daemons::Permission::Over {
                actions,
                within: daemons_within(within),
            },
            permission::daemons::Permission::Tags { actions, within, tags } => daemons::Permission::Tags {
                actions,
                within: daemons_within(within),
                tags,
            },
        }),
        permission::Grant::Accounts(permission) => Grant::Accounts(match permission {
            permission::accounts::Permission::Make(makes) => accounts::Permission::Make(makes),
            permission::accounts::Permission::Over { actions, within } => accounts::Permission::Over {
                actions,
                within: accounts_within(within),
            },
            permission::accounts::Permission::Tags { actions, within, tags } => accounts::Permission::Tags {
                actions,
                within: accounts_within(within),
                tags,
            },
        }),
        permission::Grant::Roles(permission) => Grant::Roles(match permission {
            permission::roles::Permission::Make(makes) => roles::Permission::Make(makes),
            permission::roles::Permission::Over { actions, within } => roles::Permission::Over {
                actions,
                within: roles_within(within),
            },
            permission::roles::Permission::Tags { actions, within, tags } => roles::Permission::Tags {
                actions,
                within: roles_within(within),
                tags,
            },
        }),
        permission::Grant::Volumes(permission) => Grant::Volumes(match permission {
            permission::volumes::Permission::Make(makes) => volumes::Permission::Make(makes),
            permission::volumes::Permission::Over { actions, within } => volumes::Permission::Over {
                actions,
                within: volumes_within(within),
            },
            permission::volumes::Permission::Tags { actions, within, tags } => volumes::Permission::Tags {
                actions,
                within: volumes_within(within),
                tags,
            },
        }),
        permission::Grant::Vault(permission) => Grant::Vault(match permission {
            permission::vault::Permission::Make(makes) => vault::Permission::Make(makes),
            permission::vault::Permission::Over { actions, within } => vault::Permission::Over {
                actions,
                within: vault_within(within),
            },
            permission::vault::Permission::Tags { actions, within, tags } => vault::Permission::Tags {
                actions,
                within: vault_within(within),
                tags,
            },
        }),
        permission::Grant::Postgres(permission) => Grant::Postgres(permission),
        permission::Grant::Events(permission) => Grant::Events(permission),
        permission::Grant::Images(permission) => Grant::Images(match permission {
            permission::images::Permission::Over { actions, within } => images::Permission::Over {
                actions,
                within: images_within(within),
            },
        }),
        permission::Grant::Identities(permission) => Grant::Identities(match permission {
            permission::identities::Permission::Make(makes) => identities::Permission::Make(makes),
            permission::identities::Permission::Over { actions, within } => identities::Permission::Over {
                actions,
                within: identities_within(within),
            },
            permission::identities::Permission::Tags { actions, within, tags } => identities::Permission::Tags {
                actions,
                within: identities_within(within),
                tags,
            },
        }),
    })
}

impl<T> Within<T> {
    /// `"any"` as it is, and what is named carried through `f`.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Within<U> {
        match self {
            Within::Any => Within::Any,
            Within::Only(only) => Within::Only(f(only)),
        }
    }
}

/// The agents that have never been active, as the daemon's own agents
/// filter states them: what `never_active` reaches.
fn never_active() -> crate::daemon::endpoints::agents::list::client::request::Filter {
    use crate::daemon::endpoints::agents::list::client::request::{Active, Filter, History};
    Filter {
        active: Some(Active::History(History::Never)),
        ..Default::default()
    }
}

/// A tool's reach as the daemon's `within`: `"any"` as it is, a list
/// as the filter `listed` makes of it, and tags as the filter `tags`
/// makes of them. The two filters are never one: a reach is a list or
/// tags and not both, so no filter made here names records and tags
/// together.
fn reach<L, F>(within: permission::Reach<L>, listed: impl FnOnce(L) -> F, tags: impl FnOnce(Tags) -> F) -> Within<F> {
    match within {
        permission::Reach::Any => Within::Any,
        permission::Reach::Listed(list) => Within::Only(listed(list)),
        permission::Reach::Tags(test) => Within::Only(tags(test)),
    }
}

fn agents_within(within: permission::Reach<permission::agents::Listed>) -> Within<crate::daemon::endpoints::agents::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::agents::list::client::request::Filter {
            agents: listed.agents,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::agents::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn agents_templates_within(within: permission::Reach<permission::agents_templates::Listed>) -> Within<crate::daemon::endpoints::agents::templates::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::agents::templates::list::client::request::Filter {
            ids: listed.ids,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::agents::templates::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn standalone_tools_within(within: permission::Reach<permission::standalone_tools::Listed>) -> Within<crate::daemon::endpoints::standalone_tools::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::standalone_tools::list::client::request::Filter {
            tools: listed.tools,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::standalone_tools::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn standalone_tools_templates_within(within: permission::Reach<permission::standalone_tools_templates::Listed>) -> Within<crate::daemon::endpoints::standalone_tools::templates::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::standalone_tools::templates::list::client::request::Filter {
            ids: listed.ids,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::standalone_tools::templates::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn connected_standalone_tools_within(within: permission::Reach<permission::connected_standalone_tools::Listed>) -> Within<crate::daemon::endpoints::connected_standalone_tools::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::connected_standalone_tools::list::client::request::Filter {
            tools: listed.tools,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::connected_standalone_tools::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn dependency_tools_within(within: permission::Reach<permission::dependency_tools::Listed>) -> Within<crate::daemon::endpoints::dependency_tools::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::dependency_tools::list::client::request::Filter {
            tools: listed.tools,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::dependency_tools::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn accounts_within(within: permission::Reach<permission::accounts::Listed>) -> Within<crate::daemon::endpoints::accounts::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::accounts::list::client::request::Filter {
            identities: listed.identities,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::accounts::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn roles_within(within: permission::Reach<permission::roles::Listed>) -> Within<crate::daemon::endpoints::roles::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::roles::list::client::request::Filter {
            names: listed.names,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::roles::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn daemons_within(within: permission::Reach<permission::daemons::Listed>) -> Within<crate::daemon::endpoints::daemons::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::daemons::list::client::request::Filter {
            ids: listed.ids,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::daemons::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn identities_within(within: permission::Reach<permission::identities::Listed>) -> Within<crate::daemon::endpoints::identities::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::identities::list::client::request::Filter {
            identities: listed.identities,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::identities::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn volumes_within(within: permission::Reach<permission::volumes::Listed>) -> Within<crate::daemon::endpoints::volumes::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::volumes::list::client::request::Filter {
            volumes: listed.volumes,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::volumes::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn vault_within(within: permission::Reach<permission::vault::Listed>) -> Within<crate::daemon::endpoints::vault::list::client::request::Filter> {
    reach(
        within,
        |listed| crate::daemon::endpoints::vault::list::client::request::Filter {
            keys: listed.keys,
            ..Default::default()
        },
        |tags| crate::daemon::endpoints::vault::list::client::request::Filter {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn providers_within(within: permission::Reach<permission::providers::Listed>) -> Within<providers::Reach> {
    reach(
        within,
        |listed| providers::Reach {
            identities: listed.identities,
            ..Default::default()
        },
        |tags| providers::Reach {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn providers_daemons_within(within: permission::Reach<permission::providers_daemons::Listed>) -> Within<providers::Reach> {
    reach(
        within,
        |listed| providers::Reach {
            identities: listed.identities,
            ..Default::default()
        },
        |tags| providers::Reach {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}

fn images_within(within: permission::Reach<permission::images::Listed>) -> Within<providers::Reach> {
    reach(
        within,
        |listed| providers::Reach {
            identities: listed.identities,
            ..Default::default()
        },
        |tags| providers::Reach {
            all_tags: tags.all_tags,
            any_tags: tags.any_tags,
            ..Default::default()
        },
    )
}
