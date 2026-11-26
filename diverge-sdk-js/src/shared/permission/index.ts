// The grants a tool's program declares for itself at its registration,
// and the actions every grant of the daemon's is made of.
//
// A GRANT is one permission over one kind of thing the daemon holds,
// and on the wire it is one object with one member, named for the
// kind, whose value is the kind's permission: `{"agents":…}`. The
// shapes are the daemon's own — to make, over what exists, tagging,
// the one thing. How far a grant over what exists reaches is its
// `within`, a `Reach`, one of three and never two: `"any"`, every one;
// the kind's own `Listed`, exactly the records it names; or a `Tags`
// test. Over agents alone there are the words `"attached"` and
// `"never_active"` beside. A list is read out of the arguments the
// program was registered with, never a constant of the image. Each
// kind's `Make` and `Over` live here, and the daemon's grants are
// built of them.

export * from "./grant.js";
export * from "./reach.js";
export * from "./tags.js";
export * from "./tagging.js";
export * from "./within.js";

export * as accounts from "./accounts/index.js";
export * as agents from "./agents/index.js";
export * as agents_templates from "./agents_templates/index.js";
export * as connected_standalone_tools from "./connected_standalone_tools/index.js";
export * as daemons from "./daemons/index.js";
export * as dependency_tools from "./dependency_tools/index.js";
export * as events from "./events/index.js";
export * as identities from "./identities/index.js";
export * as images from "./images/index.js";
export * as postgres from "./postgres/index.js";
export * as postgres_compartments from "./postgres_compartments/index.js";
export * as providers from "./providers/index.js";
export * as providers_daemons from "./providers_daemons/index.js";
export * as roles from "./roles/index.js";
export * as standalone_tools from "./standalone_tools/index.js";
export * as standalone_tools_templates from "./standalone_tools_templates/index.js";
export * as vault from "./vault/index.js";
export * as volumes from "./volumes/index.js";
