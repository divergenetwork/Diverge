// An agent template, as a get or a list answers it.

import { z } from "zod";
import { shown } from "../../../template/template.js";
import { AgentType } from "./agent_type.js";

/** An agent template as an answer carries it: the daemon's one `shown`, typed `AgentType`; its `arguments` only where the request asked for them. */
export const Shown = shown(AgentType);
export type Shown = z.infer<typeof Shown>;
