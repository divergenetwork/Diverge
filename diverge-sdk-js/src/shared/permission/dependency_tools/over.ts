// What is done to dependency tools that exist.

import { z } from "zod";

/**
 * The actions over dependency tools that exist, which a grant reaches as far as its `within` says.
 * Snake case on the wire: `"get"`, `"delete"`, `"list"`, `"download"`, `"upload"`, `"transfer"`, `"filetree"`, `"exec"`, `"frontend"`, `"logs"`, `"request"`.
 *
 * - `get`: Get one, as `dependency_tools.get` does.
 * - `delete`: Delete every tool of a template, as `dependency_tools.delete` does.
 * - `list`: List them, as `dependency_tools.list` does; the list sends what the grant reaches.
 * - `download`: Send the client files out of one, running, as `dependency_tools.download` does.
 * - `upload`: Put files into one, running, as `dependency_tools.upload` does, and as a transfer landing in one does.
 * - `transfer`: Copy files out of one, running, as `dependency_tools.transfer` does; where they land is judged by its own grant.
 * - `filetree`: Watch one's container whole, as `dependency_tools.filetree` does.
 * - `exec`: Run a command in one's container — any program it holds, as the container's own user — as `dependency_tools.exec` does.
 * - `frontend`: Send HTTP requests to the frontend its program serves, as `dependency_tools.frontend` does.
 * - `logs`: Read one's log, running or not, as `dependency_tools.logs` does.
 * - `request`: Make a request of them, as `requests.create` does: a request is made of a tool it addresses — by itself, by a tag, or as one of all — only where the grant reaches it.
 */
export const Over = z.enum(["get", "delete", "list", "download", "upload", "transfer", "filetree", "exec", "frontend", "logs", "request"]);
export type Over = z.infer<typeof Over>;
