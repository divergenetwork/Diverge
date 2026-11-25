// What an agent or a tool is made from, less its name, its provider
// and its mounts.

import { z } from "zod";
import { bytewise, sha256 } from "../../shared/canonical/index.js";
import { Image, hashed as hashedImage } from "../endpoints/agents/create/client/request/image.js";

/** The members every template has, beside its `type`. */
export const members = {
  /** The image: a digest and its references; the id hashes it with the references absent. */
  image: Image,
  /** How much memory the container has beyond what its image says it needs, in BYTES. The limit, a ceiling and not a hint, is the image's own `memory` and this, summed. */
  extra_memory: z.number().int().nonnegative(),
  /** How much the container may WRITE beyond what its image says it needs, in BYTES. The limit, on its own filesystem only, is the image's own `disk` and this, summed. */
  extra_disk: z.number().int().nonnegative(),
  /** What the image is told once, as the image defines it: any JSON, handed to the container and not read here. */
  arguments: z.json(),
  /** The secrets a container made from the template needs, by name: all of those its image's document states, or some, and none it does not. Required; empty for none. */
  secrets: z.array(z.string()),
};

/**
 * Everything an agent or a tool is made from that is the same for
 * every one made from it, with `type` — what the family says of
 * itself — flat among the members: the generic of the Rust, as a
 * function of the type schema. An agent template is
 * `{"type":"agent",…}` and a tool template `{"type":"tool",…}`,
 * neither decoding as the other.
 */
export function template<T extends z.ZodRawShape>(type: z.ZodObject<T>) {
  return z.object({ ...type.shape, ...members });
}

/** A template of any type, as `hashed` and `id` take one: what every template has, whatever its `type`. */
export interface Template {
  readonly image: Image;
  readonly secrets: string[];
  readonly [member: string]: unknown;
}

/**
 * The template as its id hashes it: the same, with the image's
 * references absent and the secrets' names sorted bytewise.
 */
export function hashed<T extends Template>(template: T): T {
  return { ...template, image: hashedImage(template.image), secrets: [...template.secrets].sort(bytewise) };
}

/** The template's id: sixty-four lowercase hexadecimal characters. */
export function id(template: Template): Promise<string> {
  return sha256(hashed(template));
}
