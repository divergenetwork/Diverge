// A volume of the pinned provider, mounted into the container.

import { z } from "zod";
import { Mode } from "../../../../../../provider/endpoints/volumes/mode.js";

/**
 * A volume of the provider the container is pinned to, mounted into
 * it: host side first — the volume by name, and how far into it to
 * start — then the container side. A caller cannot escape upward:
 * the offset is components, and `..` is a name, not an instruction.
 * What becomes of the changes is the volume's own mode; a mount only
 * states which mode it means the volume to have. `overlay_disk` is
 * present exactly when the mode is `ephemeral`, as a FUSE mount's is.
 */
export const VolumeMount = z
  .object({
  /** Which volume, by the name the provider's listing gives it. Naming it takes the `mount` grant over it. */
  volume_name: z.string(),
  /** How far into that volume to start, as path components relative to it. Empty mounts the volume itself. */
  volume_relative_path: z.array(z.string()),
  /** The mode the volume is meant to be in; a volume in another mode refuses the run. */
  volume_mode: Mode,
  /**
   * The most bytes the mount's own layer may hold, for an ephemeral
   * volume: what the container writes into the volume, kept apart and
   * discarded with the container. The provider sets the bytes aside
   * for the container's life, out of the one capacity the container's
   * `disk` is set aside from, and refuses the run when it cannot; a
   * write that would take the layer past them fails. `0` is a volume
   * nothing can be written into.
   */
  overlay_disk: z.number().int().nonnegative().optional(),
  /** Where it appears inside the container, as path components from the root. Empty is refused. */
  container_path: z.array(z.string()),
  })
  .refine((mount) => (mount.overlay_disk !== undefined) === (mount.volume_mode === "ephemeral"), {
    message: "overlay_disk is present exactly when volume_mode is ephemeral",
    path: ["overlay_disk"],
  });
export type VolumeMount = z.infer<typeof VolumeMount>;
