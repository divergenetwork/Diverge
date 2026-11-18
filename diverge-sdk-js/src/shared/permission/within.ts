// Every one, or only these.

import { z } from "zod";

/**
 * How far a grant reaches, or which tags it may touch: every one
 * there is, or only what `only` names. Flat on the wire: the string
 * `"any"`, or the value as it is — a filter object, a list of tags.
 * `"any"` is no filter and no list, so the two shapes never collide;
 * a string that is not `"any"` does not decode. The generic of the
 * Rust, as a function of the schema it is over.
 */
export function within<T extends z.ZodType>(only: T) {
  return z.union([z.literal("any"), only]);
}
export type Within<T> = "any" | T;

/**
 * A filter as a grant's `within`, held to the rule every reach keeps:
 * a grant reaches all, exactly the records it names, or by tags, and
 * never two of them. `naming` are the filter's members that name
 * records. A value that gives one of them together with `all_tags` or
 * `any_tags`, or gives two of them, does not decode; the filter's
 * other members narrow whichever it is. A list request's filter is
 * the same schema without this, and may give any of them together.
 */
export function only<T extends z.ZodType>(filter: T, naming: readonly string[]) {
  return filter.refine(
    (value) => {
      const given = (member: string) => {
        const list = (value as Record<string, unknown>)[member];
        return Array.isArray(list) && list.length > 0;
      };
      const named = naming.filter(given).length;
      return named <= 1 && !(named === 1 && (given("all_tags") || given("any_tags")));
    },
    { message: "a `within` reaches exactly the records it names, or by tags, never both, and names records by one member" },
  );
}
