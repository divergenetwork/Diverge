// What is done to tools that exist.

import { z } from "zod";

/**
 * The actions over tools that exist, which a grant reaches as far as its `within` says.
 * Snake case on the wire: `"get"`, `"edit"`, `"attach"`, `"detach"`, `"delete"`, `"list"`, `"download"`, `"upload"`, `"transfer"`, `"filetree"`, `"exec"`, `"frontend"`, `"api"`, `"connect"`, `"logs"`, `"request"`.
 *
 * - `get`: Get one, as `standalone_tools.get` does.
 * - `edit`: Change one, as `standalone_tools.edit` does.
 * - `attach`: Attach one to an agent, as `attach_agent_instance` does, the agent judged by an `edit` grant over agents; or to every agent of a template, as `attach_agent_template` does, the template judged by an `attach` grant over agent templates.
 * - `detach`: Detach one from an agent, as `detach_agent_instance` does; or from every agent of a template, as `detach_agent_template` does — judged as `attach` is.
 * - `delete`: Delete one, as `standalone_tools.delete` does.
 * - `list`: List them, as `standalone_tools.list` does; the list sends what the grant reaches.
 * - `download`: Send the client files out of one, as `standalone_tools.download` does.
 * - `upload`: Put files into one, as `standalone_tools.upload` does, and as a transfer landing in one does.
 * - `transfer`: Copy files out of one, as `standalone_tools.transfer` does; where they land is judged by its own grant.
 * - `filetree`: Watch one's container whole, as `standalone_tools.filetree` does.
 * - `exec`: Run a command in one's container — any program it holds, as the container's own user — as `standalone_tools.exec` does.
 * - `frontend`: Send HTTP requests to the frontend its program serves, as `standalone_tools.frontend` does.
 * - `api`: Send HTTP requests to the API its program serves, as `standalone_tools.api` does.
 * - `connect`: Serve one to another daemon, as `standalone_tools.connect` does.
 * - `logs`: Read one's log, as `standalone_tools.logs` does.
 * - `request`: Make a request of them, as `requests.create` does: a request is made of a tool it addresses — by itself, by a tag, or as one of all — only where the grant reaches it.
 */
export const Over = z.enum(["get", "edit", "attach", "detach", "delete", "list", "download", "upload", "transfer", "filetree", "exec", "frontend", "api", "connect", "logs", "request"]);
export type Over = z.infer<typeof Over>;
