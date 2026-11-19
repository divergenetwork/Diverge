// The one channel a client opens on a create: the cancel.

/**
 * What a client asks the daemon for while its request is pending: one
 * thing, `cancel`, tag `0`, carrying nothing. Nothing comes back on
 * this channel; what comes back is the scope's finish: the request is
 * withdrawn from every listener it was sent to, and the scope finishes
 * with no answer. A cancel that arrives after an answer was taken
 * changes nothing: the answer precedes the finish.
 */
export type Frame = "cancel";

/** Tag for `cancel`. */
const CANCEL = 0;

/** One byte, laid out by hand: one tag is not a shape a format would help with. */
export function encode(_frame: Frame): Uint8Array {
  return Uint8Array.of(CANCEL);
}
