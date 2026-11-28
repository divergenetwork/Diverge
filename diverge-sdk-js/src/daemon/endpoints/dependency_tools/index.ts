// Dependency tools: those an agent's program declared.
//
// A dependency tool is a dependency template under one owner — an
// agent, or nobody, as the template's image's scope decides — made by
// the first registration that declared it there, never by a request.
// Its container runs while something holds it, or as an eager tool
// the daemon keeps. Every tool an agent's program declares, of either
// scope, is deployed when the agent's container starts — one that
// cannot be fails the agent's start — and is held while that agent is
// being started, has a message on its way or is active; the file
// operations deploy one that is not running. No request here deletes one: an instance tool
// goes with its agent, and a template's every tool with the delete of
// the dependency template's compartment of the database.

export * as list from "./list/index.js";
export * as get from "./get/index.js";
export * as tag from "./tag/index.js";
export * as untag from "./untag/index.js";
export * as exec from "./exec/index.js";
export * as frontend from "./frontend/index.js";
export * as api from "./api/index.js";
export * as terminal from "./terminal/index.js";
export * as filetree from "./filetree/index.js";
export * as download from "./download/index.js";
export * as upload from "./upload/index.js";
export * as transfer from "./transfer/index.js";
