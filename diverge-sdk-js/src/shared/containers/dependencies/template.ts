// One dependency, as an agent's program declares it.

import { z } from "zod";
import { sha256 } from "../../canonical/index.js";
import { Image, hashed as hashedImage } from "../request/image.js";
import { Mount } from "./mount.js";

/**
 * One tool container an agent's program depends on: everything the
 * caller needs to deploy it that the tool's image does not say for
 * itself. The image is a dependency tool's, and its own document says
 * how the tool is kept running, how far it is shared, and the memory
 * and the disk it needs. Here:
 * the image, what the container is given beyond what the image needs,
 * the arguments, and the agent's files and directories served live into it,
 * which only an image that is lazy and of the `instance` scope is
 * declared with.
 */
export const Template = z.object({
  image: Image,
  extra_memory: z.number().int().nonnegative(),
  extra_disk: z.number().int().nonnegative(),
  arguments: z.json(),
  fuse_file_mounts: z.array(Mount).optional(),
  fuse_directory_mounts: z.array(Mount).optional(),
});
export type Template = z.infer<typeof Template>;

/**
 * The template as its id hashes it: the same, with the image's
 * references absent.
 */
export function hashed(template: Template): Template {
  return { ...template, image: hashedImage(template.image) };
}

/** The template's id: the lowercase hexadecimal SHA-256 of its canonical bytes. */
export function id(template: Template): Promise<string> {
  return sha256(hashed(template));
}
