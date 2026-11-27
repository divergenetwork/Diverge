// What is done to tool templates that exist.

import { z } from "zod";

/**
 * The actions over tool templates that exist, which a grant reaches as far as its `within` says.
 * Snake case on the wire: `"get"`, `"arguments"`, `"delete"`, `"list"`.
 *
 * - `get`: Get one, as `standalone_tools.templates.get` does.
 * - `arguments`: See one's arguments: have a `get` or a `list` whose request says `"arguments": true` answered with them. Held beside `get` or `list` and never in place of either: alone it answers nothing.
 * - `delete`: Delete one, as `standalone_tools.templates.delete` does.
 * - `list`: List them, as `standalone_tools.templates.list` does; the list sends what the grant reaches.
 */
export const Over = z.enum(["get", "arguments", "delete", "list"]);
export type Over = z.infer<typeof Over>;
