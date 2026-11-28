// What opens an API request: its method, its path and its headers —
// a frontend request's own form, beneath another prefix.

/**
 * One HTTP request's head, as `shared.containers.frontend.request.Head`
 * states it: the method, the path with its query — `/api` or beneath
 * it — and the headers in their order.
 */
export { Head } from "../../frontend/request/head.js";
