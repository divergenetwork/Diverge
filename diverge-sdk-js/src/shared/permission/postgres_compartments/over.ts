// What is done to compartments.

import { z } from "zod";

/**
 * The actions over compartments, which a grant reaches as far as its `within` says.
 * Snake case on the wire: `"get"`, `"list"`, `"delete"`.
 *
 * - `get`: Get one, as `postgres.compartments.get` does.
 * - `list`: List them, as `postgres.compartments.list` does; the list sends what the grant reaches.
 * - `delete`: Delete one, with every table in it, as `postgres.compartments.delete` does.
 */
export const Over = z.enum(["get", "list", "delete"]);
export type Over = z.infer<typeof Over>;
