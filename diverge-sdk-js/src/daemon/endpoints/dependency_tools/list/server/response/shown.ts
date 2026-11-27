// A dependency template, as a get or a list answers it.

import { z } from "zod";
import { Mount } from "../../../../../../shared/containers/dependencies/mount.js";
import { Image } from "../../../../../../shared/containers/request/image.js";

/**
 * A dependency `Template` as an answer carries it: every member of
 * it, the `arguments` only where the request asked for them, which
 * takes the `arguments` action over the tool. Without them it is not
 * the bytes the id is the hash of. An answer's shape alone: what a
 * program declares is a `Template`, whose arguments are required.
 */
export const Shown = z.object({
  /** The image: a digest and its references. */
  image: Image,
  /** The template's `extra_memory`, in bytes. */
  extra_memory: z.number().int().nonnegative(),
  /** The template's `extra_disk`, in bytes. */
  extra_disk: z.number().int().nonnegative(),
  /** The template's arguments, where the request said `"arguments": true`. Absent where it said `false`: absent is not shown, and never a template without arguments, which has none. */
  arguments: z.json().optional(),
  /** The template's `fuse_file_mounts`. Absent when empty. */
  fuse_file_mounts: z.array(Mount).optional(),
  /** The template's `fuse_directory_mounts`. Absent when empty. */
  fuse_directory_mounts: z.array(Mount).optional(),
});
export type Shown = z.infer<typeof Shown>;
