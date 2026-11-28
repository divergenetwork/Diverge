// The dependencies a container declares, asked of the caller: each a
// tool container, as a template the caller deploys unread.
//
// An agent container's program answers its registration with the
// tools it depends on — each a `Template`: a tool container the
// caller runs, by its image, its limits and its arguments; which of two modes it is, a lazy
// tool run while anything holds it or an eager one held as well by its attachments; the grants
// its account holds — and named by its id, the hash of its canonical
// bytes. A tool container has no dependencies.

export * from "./template.js";
export * from "./mount.js";
