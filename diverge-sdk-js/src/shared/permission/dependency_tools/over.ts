// What is done to dependency tools that exist.

import { z } from "zod";

/**
 * The actions over dependency tools that exist, which a grant reaches as far as its `within` says.
 * Snake case on the wire: `"get"`, `"list"`, `"arguments"`, `"download"`, `"upload"`, `"transfer"`, `"filetree"`, `"exec"`, `"frontend"`, `"api"`, `"logs"`, `"request"`.
 *
 * - `get`: Get one, as `dependency_tools.get` does.
 * - `list`: List them, as `dependency_tools.list` does; the list sends what the grant reaches.
 * - `arguments`: See the arguments of one's template: have a `get` or a `list` whose request says `"arguments": true` answered with them. Held beside `get` or `list` and never in place of either: alone it answers nothing.
 * - `download`: Send the client files out of one, running, as `dependency_tools.download` does.
 * - `upload`: Put files into one, running, as `dependency_tools.upload` does, and as a transfer landing in one does.
 * - `transfer`: Copy files out of one, running, as `dependency_tools.transfer` does; where they land is judged by its own grant.
 * - `filetree`: Watch one's container whole, as `dependency_tools.filetree` does.
 * - `exec`: Run a command in one's container — any program it holds, as the container's own user — as `dependency_tools.exec` does.
 * - `frontend`: Send HTTP requests to the frontend its program serves, as `dependency_tools.frontend` does.
 * - `api`: Send HTTP requests to the API its program serves, as `dependency_tools.api` does.
 * - `logs`: Read one's log, running or not, as `dependency_tools.logs` does.
 * - `request`: Make a request of them, as `requests.create` does: a request is made of a tool it addresses — by itself, by a tag, or as one of all — only where the grant reaches it.
 */
export const Over = z.enum(["get", "list", "arguments", "download", "upload", "transfer", "filetree", "exec", "frontend", "api", "logs", "request"]);
export type Over = z.infer<typeof Over>;
