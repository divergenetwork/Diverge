// Asking an agent something: a chat message, or a structured
// decision, by the request's `type`. Both go to the agent in the
// order the daemon admitted them. Unwatched, the answer is the
// request's fate; watched, it is what the agent's log takes of it.
// One grant covers both: `message`, over agents.
//
// A message is what makes an agent active. It deploys the agent if
// its container is down, and with it every tool the agent is served —
// the dependency tools its program declares, of either scope, lazy or
// eager, the standalone tools attached to it or to its template, and
// the connected tools — all started together and all waited for; a
// tool that cannot be deployed fails the start, naming the tool, and
// the message with it. A message for an agent that is up already
// first brings back any tool it is served that has gone down; one
// that cannot be brought back fails the message, and nothing is
// delivered. The agent holds its tools from its registration, while a
// message is on its way into it, and while it is active.

export * as client from "./client/index.js";
export * as server from "./server/index.js";
