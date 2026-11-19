// Whether a daemon announces itself on a provider, and with what.

import { z } from "zod";
import { Description } from "./description.js";

/**
 * What a provider's record says of the daemon announcing itself on it.
 * Untagged on the wire: a boolean, or a string.
 *
 * - `false`: the daemon does not announce itself on the provider, and
 *   no other daemon finds it there.
 * - `true`: it announces itself, with no text.
 * - a string: it announces itself with that text, a `Description` — one
 *   line of plain text, 1 to 132 characters, trimmed. The empty string
 *   is none: an announcement with no text is `true`.
 */
export const Announcement = z.union([z.boolean(), Description]);
export type Announcement = z.infer<typeof Announcement>;
