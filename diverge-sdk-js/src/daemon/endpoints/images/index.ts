// Images: named by digest, never held. `read` asks whether any source
// the daemon reaches can supply the image, and, where one can,
// everything read of it, through it: its manifests, its blobs, its
// proof, and its DIVERGE.md. `build` makes one on a provider, from a
// Containerfile, into the daemon's Store there; `push` puts one there
// that is built already, from a registry the provider uses or from
// the client; `list` is what the daemon owns in its Stores, `get`
// one image of them, `delete` one of them taken out, or some of
// its platforms, and `publish` one or more sent to a registry as one
// image.

export * as build from "./build/index.js";
export * as delete from "./delete/index.js";
export * as get from "./get/index.js";
export * as list from "./list/index.js";
export * as merge from "./merge/index.js";
export * as publish from "./publish/index.js";
export * as push from "./push/index.js";
export * as read from "./read/index.js";
