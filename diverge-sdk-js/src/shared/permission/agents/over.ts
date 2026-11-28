// What is done to agents that exist.

import { z } from "zod";

/**
 * The actions over agents that exist, which a grant reaches as far as its `within` says.
 * Snake case on the wire: `"get"`, `"delete"`, `"edit"`, `"message"`, `"logs"`, `"list"`, `"download"`, `"upload"`, `"transfer"`, `"filetree"`, `"exec"`, `"connect"`, `"request"`.
 *
 * - `get`: Get one, as `agents.get` does.
 * - `delete`: Delete one, as `agents.delete` does.
 * - `edit`: Change one, as `agents.edit` does.
 * - `message`: Send one a message, as `agents.message` does.
 * - `logs`: Read one's log, as `agents.logs` does.
 * - `list`: List them, as `agents.list` does; the list sends what the grant reaches.
 * - `download`: Send the client files out of one, as `agents.download` does.
 * - `upload`: Put files into one, as `agents.upload` does, and as a transfer landing in one does.
 * - `transfer`: Copy files out of one, as `agents.transfer` does; where they land is judged by its own grant.
 * - `filetree`: Watch one's container whole, as `agents.filetree` does.
 * - `exec`: Run a command in one's container — any program it holds, as the container's own user — as `agents.exec` does.
 * - `connect`: Run MCP exchanges as one — list and call its tools, list and read its resources, as its own container would — as `agents.connect` does: the scope deploys its container and holds it up while it is open, and makes it neither active nor busy.
 * - `request`: Make a request of them, as `requests.create` does: a request is made of an agent it addresses — by itself, by a tag, or as one of all — only where the grant reaches it.
 */
export const Over = z.enum(["get", "delete", "edit", "message", "logs", "list", "download", "upload", "transfer", "filetree", "exec", "connect", "request"]);
export type Over = z.infer<typeof Over>;
