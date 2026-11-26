// Grants over the database's compartments.
//
// `Permission` is one grant's worth over compartments, reaching by the
// family's own list filter; the actions are `shared.permission`'s.

export * from "./permission.js";
export { Over } from "../../../shared/permission/postgres_compartments/index.js";
