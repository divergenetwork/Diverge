// Which of the two modes a dependency template declares.

import { z } from "zod";
import { Mount } from "./mount.js";
import { Scope } from "./scope.js";

/** A tool run while it is used: `"mode":"lazy"`, with how far it is shared and, at the `instance` scope alone, the agent's paths it mounts. */
export const Lazy = z.object({
  mode: z.literal("lazy"),
  /** How far the tool is shared: see `Scope`. Required. */
  scope: Scope,
  /** Files of the agent container served live into the tool's, one mount each. Only where `scope` is `instance`. Absent when none. */
  fuse_file_mounts: z.array(Mount).optional(),
  /** Directories of the agent container served live into the tool's, one mount each. Only where `scope` is `instance`. Absent when none. */
  fuse_directory_mounts: z.array(Mount).optional(),
});
export type Lazy = z.infer<typeof Lazy>;

/** A tool the caller keeps: `"mode":"eager"`, with how far it is shared. */
export const Eager = z.object({
  mode: z.literal("eager"),
  /** How far the tool is shared: see `Scope`. Required. */
  scope: Scope,
});
export type Eager = z.infer<typeof Eager>;

/**
 * Why a mode is not one a caller deploys, or `undefined` where it
 * is. A lazy tool whose scope is `template` or `global` names no
 * mount: it is many agents', and there is no one agent whose files a
 * mount would be. An eager tool has no mounts to name.
 */
export function refusal(mode: { mode: string; scope: Scope; fuse_file_mounts?: unknown[] | undefined; fuse_directory_mounts?: unknown[] | undefined }): string | undefined {
  if (mode.mode !== "lazy" || mode.scope === "instance") return undefined;
  const mounts = (mode.fuse_file_mounts?.length ?? 0) + (mode.fuse_directory_mounts?.length ?? 0);
  return mounts === 0 ? undefined : "a lazy tool mounts the agent's files only where its scope is `instance`";
}

/**
 * A lazy tool, or an eager one: the two things a dependency can be,
 * told apart on the wire by the `mode` member beside the template's
 * other members, with the members of that mode flat beside it. Both
 * say a `scope`. A lazy tool runs while it is used: at the
 * `instance` scope it is deployed when the agent's container starts
 * and stopped with the agent's run; at `template` or `global` it is
 * one container for every agent the scope covers, started by its
 * first use and stopped once idle. An eager tool is made once, under
 * the scope key its `Scope` names, and kept by the caller for as
 * long as any agent is attached to it.
 */
export const Mode = z.discriminatedUnion("mode", [Lazy, Eager]).superRefine((mode, context) => {
  const reason = refusal(mode);
  if (reason !== undefined) context.addIssue({ code: "custom", message: reason });
});
export type Mode = z.infer<typeof Mode>;
