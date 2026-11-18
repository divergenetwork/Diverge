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
 *   from. Every provider connected, the one pushed to among them, and
 *   every remote daemon on record is asked at once which platforms of
 *   the image it can supply. Once every platform named has a source,
 *   the daemon puts one push to the provider for each source, naming
 *   that source's platforms, all at once: an image whose platforms are
 *   spread over several sources is pushed from all of them. A platform
 *   no source supplies is `unavailable`, and nothing moved.
 *
 * Neither needs a grant beyond `push` over images reaching the
 * provider pushed to. A source that stops answering is answered to the
 * provider as not held: the push it was answering is `unavailable`.
 */
export const Source = z.enum(["client", "daemon"]);
export type Source = z.infer<typeof Source>;
