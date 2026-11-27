//! Judging requests over the database's compartments.
//!
//! The shape of [`accounts`](super::accounts), with no making action: an
//! action over a compartment is allowed by a grant that holds it and
//! whose `within` reaches the compartment — `any`, or the compartments
//! list filter as [`filter::postgres_compartments::test`] reads it; a
//! tagging action by a grant that holds it, reaches the compartment,
//! and covers every tag named.

use diverge_sdk::daemon::endpoints::postgres::compartments::list::client::request::Filter;
use diverge_sdk::daemon::grant::postgres_compartments::{Over, Permission};
use diverge_sdk::daemon::grant::{Tagging, Within};

use super::Standing;
use super::filter;
use super::filter::postgres_compartments::Facts;

/// Whether the standing holds `action` over any compartment at all.
pub fn holds(standing: &Standing, action: Over) -> bool {
    if standing.admin() {
        return true;
    }
    standing
        .postgres_compartments()
        .any(|permission| matches!(permission, Permission::Over { actions, .. } if actions.contains(&action)))
}

/// Whether the standing may do `action` to the compartment.
pub fn over(standing: &Standing, action: Over, facts: &Facts<'_>) -> bool {
    if standing.admin() {
        return true;
    }
    standing.postgres_compartments().any(|permission| match permission {
        Permission::Over { actions, within } => actions.contains(&action) && reaches(within, facts),
        _ => false,
    })
}

/// Whether the standing holds the tagging `action` over any
/// compartment at all.
pub fn holds_tagging(standing: &Standing, action: Tagging) -> bool {
    if standing.admin() {
        return true;
    }
    standing
        .postgres_compartments()
        .any(|permission| matches!(permission, Permission::Tags { actions, .. } if actions.contains(&action)))
}

/// Whether the standing may put `tags` on the compartment, or take
/// them off, as `action` says: one grant must hold the action, reach
/// the compartment, and name every one of the tags or `any`.
pub fn tagging(standing: &Standing, action: Tagging, facts: &Facts<'_>, tags: &[String]) -> bool {
    if standing.admin() {
        return true;
    }
    standing.postgres_compartments().any(|permission| match permission {
        Permission::Tags {
            actions,
            within,
            tags: scope,
        } => actions.contains(&action) && reaches(within, facts) && covers(scope, tags),
        _ => false,
    })
}

/// Whether the grant's reach takes in the compartment.
fn reaches(within: &Within<Filter>, facts: &Facts<'_>) -> bool {
    match within {
        Within::Any => true,
        Within::Only(filter) => filter::postgres_compartments::test(filter, facts),
    }
}

/// Whether the grant's tags take in every one named.
fn covers(scope: &Within<Vec<String>>, tags: &[String]) -> bool {
    match scope {
        Within::Any => true,
        Within::Only(covered) => tags.iter().all(|tag| covered.contains(tag)),
    }
}
