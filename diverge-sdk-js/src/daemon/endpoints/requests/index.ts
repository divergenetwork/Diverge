// Requests: a question one party puts to others, and the one answer
// it gets. `create` makes one of accounts, agents and tools, each by itself or
// by a tag, or of all the caller may ask; every one of them that
// `listen`s is sent it; the first answer the daemon takes is the
// request's, and every other listener it was sent to is told it is
// withdrawn. A request lives with its creator's scope and is kept
// nowhere else.

export * from "./id.js";
export * as create from "./create/index.js";
export * as listen from "./listen/index.js";
