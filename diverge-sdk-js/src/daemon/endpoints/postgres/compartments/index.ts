// The compartments of the database: the three a container reaches,
// listed, tagged and deleted.
//
// A container's driver names one of three databases, `instance`,
// `template` or `image`, and each is a `Compartment`: one login role
// and one schema in the daemon's one database. An agent's or a tool's
// `instance` goes with its delete, and its `template` with the
// template's; a dependency tool's `instance` goes with its agent's.
// A dependency template's `template` and every `image` go by `delete`
// alone, and only while nothing keeps them: each item's `deletable`
// says whether one would be allowed now.

export * from "./compartment.js";
export * as list from "./list/index.js";
export * as get from "./get/index.js";
export * as tag from "./tag/index.js";
export * as untag from "./untag/index.js";
export * as delete from "./delete/index.js";
