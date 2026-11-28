// What brought a run's container up.

import { z } from "zod";

/**
 * What deployed a run, as its `deployed` says. Exactly one per deploy:
 * the one thing that found the container down and started it. What
 * uses the container after that is not told here.
 *
 * - `message`: a message for the agent, which starts a loop. An
 *   agent's alone.
 * - `connect`: an MCP exchange on a connect scope: on an agent's, one
 *   run as the agent; on a standalone tool's, the scope's opening.
 * - `registration`: an agent's container started and its program
 *   declared the tool at its registration. A dependency tool's alone.
 * - `agent_active`: an agent the tool is attached to began a loop. A
 *   tool's alone.
 * - `attachment`: an agent came to be attached to an eager tool. A
 *   tool's alone.
 * - `restart`: an eager tool that is still held was started again:
 *   after its container ended, or at the daemon's start. A tool's
 *   alone.
 * - `dependency_tool`: a dependency tool of the agent's that mounts
 *   the agent's files was deployed while the agent's container was
 *   down. An agent's alone.
 * - `download`, `upload`, `transfer`, `filetree`, `exec`, `terminal`,
 *   `frontend`: that request into the container.
 */
export const Cause = z.enum([
  "message",
  "connect",
  "registration",
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
]);
export type Cause = z.infer<typeof Cause>;
