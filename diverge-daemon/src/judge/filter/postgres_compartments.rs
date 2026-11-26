//! The compartments filter, as a test.

use diverge_sdk::daemon::endpoints::postgres::Container;
use diverge_sdk::daemon::endpoints::postgres::compartments::Compartment;
use diverge_sdk::daemon::endpoints::postgres::compartments::list::client::request::Filter;
use diverge_sdk::daemon::reference;

use crate::store::postgres_compartments::Entry;

/// What a compartment is judged by: its record, and whether a delete
/// would be allowed now, which is no column of it.
#[derive(Debug, Clone, Copy)]
pub struct Facts<'a> {
    /// The record.
    pub entry: &'a Entry,
    /// Whether a delete would be allowed now, connections apart.
    pub deletable: bool,
}

/// The template the compartment is of, by id: a `template`
/// compartment's own, an agent's or a tool's `instance`'s, a
/// dependency tool's `instance`'s dependency template. An `image`
/// compartment is of none, and an agent named by name is not how the
/// daemon names one.
fn template(compartment: &Compartment) -> Option<&str> {
    match compartment {
        Compartment::Template { template, .. } => Some(template),
        Compartment::Instance(Container::Agent(reference::Agent::TemplateIndex { template, .. })) => Some(template),
        Compartment::Instance(Container::Tool(reference::Tool::TemplateIndex { template, .. })) => Some(template),
        Compartment::Instance(Container::Dependency(reference::Dependency { template, .. })) => Some(template),
        _ => None,
    }
}

/// Whether the compartment passes `filter`.
pub fn test(filter: &Filter, facts: &Facts<'_>) -> bool {
    let entry = facts.entry;
    (filter.compartments.is_empty() || filter.compartments.contains(&entry.compartment))
        && (filter.databases.is_empty() || filter.databases.contains(&entry.compartment.database()))
        && (filter.kinds.is_empty() || filter.kinds.contains(&entry.kind))
        && (filter.templates.is_empty() || template(&entry.compartment).is_some_and(|id| filter.templates.iter().any(|wanted| wanted == id)))
        && (filter.images.is_empty() || matches!(&entry.compartment, Compartment::Image { digest } if filter.images.contains(digest)))
        && filter.deletable.is_none_or(|deletable| deletable == facts.deletable)
        && filter.all_tags.iter().all(|tag| entry.tags.contains(tag))
        && (filter.any_tags.is_empty() || filter.any_tags.iter().any(|tag| entry.tags.contains(tag)))
        && filter.created_from.is_none_or(|from| entry.created >= from)
        && filter.created_to.is_none_or(|to| entry.created <= to)
}
