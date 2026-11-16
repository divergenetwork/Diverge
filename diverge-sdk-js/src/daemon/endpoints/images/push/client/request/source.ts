// Where a pushed image's bytes come from.

import { z } from "zod";
import { Identity } from "../../../../../agent_log/identity.js";
import { Relay } from "../../../../../../shared/relay.js";

/**
 * Who answers the provider's asks for the image being pushed: its
 * manifests, its blobs and the proof that it is held. It is asked only
 * where the provider asks: not for a digest the daemon owns there
 * already, and not for one a registry the provider uses serves.
 *
 * - `"client"`: the client that sent the request. The daemon carries
 *   each ask to the client's scope and the answer back.
 * - `{ provider }`: one provider on the daemon's record. The daemon
 *   answers the asks itself, from an `images.read` scope it opens there
 *   as itself. Reached by `read` in the `providers` grant.
 * - `{ daemon }`: one remote daemon, by a route read as a request's
 *   `relay` is: the first id is this daemon's record, and the rest are
 *   the route beyond it. The daemon answers the asks itself, from an
 *   `images.read` scope it opens there. Reached by `relay` in the
 *   `daemons` grant over the first record. A route of no id is refused.
 *
 * A source that does not hold the image, or stops answering, is
 * answered to the provider as not held: the push is `unavailable`.
 */
export const Source = z.union([
  z.literal("client"),
  z.strictObject({ provider: Identity }),
  z.strictObject({ daemon: Relay }),
]);
export type Source = z.infer<typeof Source>;
