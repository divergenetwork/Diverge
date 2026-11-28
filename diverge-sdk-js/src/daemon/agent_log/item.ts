// One item of a log, of ten kinds.

import { z } from "zod";
import { AgenticLoopChunk } from "../../provider/endpoints/containers/agents/run/server/response/chunk.js";
import { Command } from "../command_log/command.js";
import { Active } from "./active.js";
import { Deployed } from "./deployed.js";
import { Error } from "./error.js";
import { Inactive } from "./inactive.js";
import { StructuredDecisionsResponse } from "./structured_decisions_response.js";
import { Undeployed } from "./undeployed.js";
import { User } from "./user.js";
import { UserStructuredDecisions } from "./user_structured_decisions.js";

/**
 * One item of a log: a user part with its sender, a chunk of the
 * loop, an error, a request the agent's own container put to the
 * daemon (`command`, as `command_log` states: it makes the agent
 * neither active nor inactive), or one step of the run's life — `deployed`,
 * `active`, `inactive`, `undeployed`, as `lifecycle` states. Untagged
 * JSON, told apart by each kind's own `type` and, for a user part,
 * by the presence of `sender`: nothing but a user part has one, so
 * `User` is tried before a chunk. A structured decision and its
 * answers are each a chunk with the log's numeric `id` added, and
 * are tried before both, which would each read one and drop the id.
 */
export const Item = z.union([
  UserStructuredDecisions,
  StructuredDecisionsResponse,
  User,
  AgenticLoopChunk,
  Error,
  Command,
  Deployed,
  Active,
  Inactive,
  Undeployed,
]);
export type Item = z.infer<typeof Item>;
