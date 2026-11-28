// A run's life, as an agent's log and a tool's log both keep it: the
// same four items in the same JSON — `deployed`, `active`, `inactive`,
// `undeployed`. A `deployed` says what deployed the run, a `Cause`.
// An agent is active while a loop runs in it or an MCP exchange its
// own container asked for is in flight, and nothing else: not a use
// of its container, and not an exchange run as it on an
// `agents.connect` scope. A tool is active while an ask is in flight.
// No `active` precedes a run's `deployed`, and no
// `inactive` follows its `undeployed`; an unexpected end is said only
// by `undeployed`'s `reason`, never by a plain `error`.

export * from "./provider.js";
export * from "./reason.js";
export * from "./cause.js";
export * from "./deployed.js";
export * from "./undeployed.js";
