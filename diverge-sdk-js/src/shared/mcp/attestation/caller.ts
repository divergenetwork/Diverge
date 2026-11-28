// Who a request came from.

import { z } from "zod";
import { Identity } from "../../identity.js";
import { Agent } from "./agent.js";
import { Tool } from "./tool.js";

/**
 * Who a request came from, as the daemon attests it on the request's
 * `_meta`, and in the `diverge.network` header of every frontend
 * request and every API request. Tagged by `kind`, three kinds, one
 * for each kind of requester a daemon has:
 *
 * 1. An agent, by its image and its key: the same caller whichever
 *    way the agent comes — an MCP call its container sent, or a
 *    connect, a frontend request or an API request it put to the
 *    daemon over its own daemon connection. It carries no scope.
 * 2. A client, by the identity its handshake established — as every
 *    `Creator.client` names the same client — and the scope's id.
 * 3. A tool container of the daemon's, reaching the tool over its own
 *    daemon connection: the tool as the daemon attests one, and the
 *    scope's id.
 *
 * Nothing a caller says of itself is trusted: every member is the
 * daemon's own word. A client's and a tool's `scope` is minted for the
 * one connect, frontend or API scope the call came on, so two scopes
 * of one caller are two callers to a tool, and two MCP sessions; an
 * agent has none, and one session with each tool.
 */
export const Caller = z.discriminatedUnion("kind", [
  /** An agent of the daemon's. */
  z.object({ kind: z.literal("agent"), ...Agent.shape }),
  /**
   * A client of the daemon's, calling over a connect, a frontend
   * request or an API request: the account the connection is, and the
   * scope's id, a random UUID the daemon minted for that one scope.
   */
  z.object({ kind: z.literal("client"), identity: Identity, scope: z.string() }),
  /**
   * A tool container of the daemon's, calling over its own daemon
   * connection by a connect, a frontend request or an API request: the
   * tool as the daemon attests one — `created` or `dependency`; a
   * connected tool runs on another daemon and never calls here — and
   * the scope's id, as a client's is.
   */
  z.object({ kind: z.literal("tool"), tool: Tool, scope: z.string() }),
]);
export type Caller = z.infer<typeof Caller>;
