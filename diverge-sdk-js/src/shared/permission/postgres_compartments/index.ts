// Grants over the database's compartments.
//
// What a dependency tool template may ask over the compartments, in
// the shapes `permission` states: `Over`, the actions over
// compartments, reaching all, those listed, or by tags; and
// `Permission`, one grant's worth of that, or of tagging. Nothing here
// makes a compartment. The daemon's grants are made of the same
// actions.

export * from "./listed.js";
export * from "./over.js";
export * from "./permission.js";
