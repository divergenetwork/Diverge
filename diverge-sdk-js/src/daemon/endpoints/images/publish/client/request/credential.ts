// What a publish writes to the registry as.

import { z } from "zod";

/**
 * The credential a publish presents to the registry: a username and a
 * password, the password whatever the registry accepts beside the
 * username — a password, an access token, a robot's secret. It is for
 * this publish alone: the daemon hands it to the provider the publish
 * is done on and keeps it nowhere.
 */
export const Credential = z.object({
  /** The username. */
  username: z.string(),
  /** The password, or the token the registry takes in its place. */
  password: z.string(),
});
export type Credential = z.infer<typeof Credential>;
