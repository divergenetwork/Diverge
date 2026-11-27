// A tool template, as a get or a list answers it.

import { z } from "zod";
import { shown } from "../../../template/template.js";
import { ToolType } from "./tool_type.js";

/** A tool template as an answer carries it: the daemon's one `shown`, typed `ToolType`; its `arguments` only where the request asked for them. */
export const Shown = shown(ToolType);
export type Shown = z.infer<typeof Shown>;
