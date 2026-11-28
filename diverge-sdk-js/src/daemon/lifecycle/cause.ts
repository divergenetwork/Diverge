// What brought a run's container up.

import { z } from "zod";

/**
 * What deployed a run, as its `deployed` says. Exactly one per deploy:
 * the one thing that found the container down and started it. What
 * uses the container after that is not told here.
 *
 * - `message`: a message for the agent, which starts a loop. An
 *   agent's alone.
 * - `connect`: a connect scope: on an agent's and on a standalone
 *   tool's, the scope's opening; and, of a tool an agent is served,
 *   an exchange run as the agent on an agent's connect scope that
 *   found the tool down.
 * - `registration`: an agent's container started and its program
 *   declared the tool at its registration. A dependency tool's alone.
 * - `agent_deployed`: an agent the tool is served to was deployed:
 *   every tool an agent is served is deployed with it. A standalone
 *   or a connected tool's; a dependency tool the agent's program
 *   declared is a `registration`.
 * - `agent_active`: a tool an agent is served had gone down while
 *   the agent's container stayed up, and was brought back for the
 *   agent: before a message was delivered to it, or by an exchange
 *   of its own container that found the tool down. A tool's alone.
 * - `attachment`: an agent came to be attached to an eager tool. A
 *   tool's alone.
 * - `restart`: an eager tool that is still held was started again:
 *   after its container ended, or at the daemon's start. A tool's
 *   alone.
 * - `dependency_tool`: a dependency tool of the agent's that mounts
 *   the agent's files was deployed while the agent's container was
 *   down. An agent's alone.
 * - `download`, `upload`, `transfer`, `filetree`, `exec`, `terminal`:
 *   that request into the container.
 * - `frontend`: a request to the frontend the tool's program serves. A
 *   tool's alone: an agent's program serves none.
 * - `api`: a request to the API the tool's program serves. A tool's
 *   alone.
 */
export const Cause = z.enum([
  "message",
  "connect",
  "registration",
  "agent_deployed",
  "agent_active",
  "attachment",
  "restart",
  "dependency_tool",
  "download",
  "upload",
  "transfer",
  "filetree",
  "exec",
  "terminal",
  "frontend",
  "api",
]);
export type Cause = z.infer<typeof Cause>;
