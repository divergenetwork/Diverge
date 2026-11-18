// Grants over vault keys that name none of them.
//
// What a dependency tool template may ask over vault keys, in the shapes
// `permission` states: `Make`, the actions that bring one into being; `Over`,
// the actions over those that exist, reaching all, those listed, or by tags; and
// `Permission`, one grant's worth of either, or of tagging. The
// daemon's grants over vault keys are made of the same actions.

export * from "./listed.js";
export * from "./make.js";
export * from "./over.js";
export * from "./permission.js";
