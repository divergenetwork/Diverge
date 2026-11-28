// MCP as an agent: a scope held open on an agent, on which the
// client opens channels — one per MCP exchange, and the disconnect —
// and the daemon opens none. The daemon answers exactly one
// `connected`, and every exchange after is answered exactly as the
// same exchange from the agent's own container is: the agent's tools
// by the names it sees them by, stamped as the agent's, logged in
// each tool's log as the agent's. Opening the scope deploys the
// agent's container if it is down — the ordinary start, which deploys
// every tool the agent is served and fails where one cannot be — and
// holds it up for as long as the scope is open, with or without an
// exchange in flight. Nothing on the scope makes the agent active or
// keeps it active, and nothing of it is kept in the agent's log. A
// list, a call or a read holds the tool it reaches while it is in
// flight, and deploys that tool if it is down. The scope ends at the
// disconnect, at the client's connection ending, or at the agent's
// container ending; a container that ended is not started again
// under the scope.

export * as client from "./client/index.js";
export * as server from "./server/index.js";
