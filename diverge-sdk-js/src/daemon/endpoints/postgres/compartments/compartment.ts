// One compartment, as a request names it and the daemon answers it.

import { z } from "zod";
import { Container } from "../container.js";

/**
 * Which of its databases a container's connection reaches: what the
 * driver names as the database. `instance` is the container's own;
 * `template` every container's made from the same template; `image`
 * every container's of the same image. A dependency tool whose
 * image's `scope` is `template` has no `instance`. Snake case on the
 * wire.
 */
export const Database = z.enum(["instance", "template", "image"]);
export type Database = z.infer<typeof Database>;

/**
 * What kind of container a compartment is of: the `type` of an
 * image's document. Snake case on the wire.
 */
export const Kind = z.enum(["agent", "standalone_tool", "dependency_tool"]);
export type Kind = z.infer<typeof Kind>;

/**
 * One compartment of the database: what a container's connection
 * reaches when its driver names `instance`, `template` or `image`.
 * Externally tagged, the member naming which of the three it is:
 * `{"instance":<container>}`, one container's own;
 * `{"template":{"kind":…,"template":…}}`, every container's made from
 * one template; `{"image":{"digest":…}}`, every container's of one
 * image. A request names an agent either way, by name or by template
 * and index; the daemon answers template and index, and a filter and
 * a grant match that form alone.
 */
export const Compartment = z.union([
  /** One container's own. A dependency tool's owner is an agent, never `"template"`. */
  z.strictObject({ instance: Container }),
  /** Every container's made from one template. */
  z.strictObject({
    template: z.strictObject({
      /** What the template makes. */
      kind: Kind,
      /** The template, by id. */
      template: z.string(),
    }),
  }),
  /** Every container's of one image. */
  z.strictObject({
    image: z.strictObject({
      /** The image, by digest. */
      digest: z.string(),
    }),
  }),
]);
export type Compartment = z.infer<typeof Compartment>;
