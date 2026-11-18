// Where a pushed image's bytes come from.

import { z } from "zod";

/**
 * Who answers the provider's asks for the image being pushed: its
 * manifests, its blobs and the proof that it is held. It is asked only
 * where the provider asks: not for a digest the daemon owns there
 * already, and not for one a registry the provider uses serves.
 *
 * - `"client"`: the client that sent the request. The daemon carries
 *   each ask to the client's scope and the answer back.
 * - `"daemon"`: the daemon answers the asks itself and picks where
 *   from, as it does for the image of a container it runs: every other
 *   provider connected and every remote daemon on record is asked at
 *   once whether it can supply the image, and the first that can
 *   supplies it. The provider pushed to is not asked.
 *
 * Neither needs a grant beyond `push` over images reaching the
 * provider pushed to. A source that does not hold the image, or stops
 * answering, is answered to the provider as not held: the push is
 * `unavailable`.
 */
export const Source = z.enum(["client", "daemon"]);
export type Source = z.infer<typeof Source>;
