// How large a file of an image's `/.diverge` may be.

/**
 * The most bytes a file read out of an image's `/.diverge` may hold:
 * `1048576`, which is 2 to the power 20. It bounds the file as the
 * image holds it, before anything is parsed or decoded, and it bounds
 * each file by itself: the document, and each icon the document
 * names. A file of exactly this many bytes is within the limit; a
 * provider answers no part of a longer one, and says that it is too
 * large.
 */
export const MAX_BYTES = 1048576;
