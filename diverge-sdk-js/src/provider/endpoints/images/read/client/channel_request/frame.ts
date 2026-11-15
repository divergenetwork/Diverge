// The channels a caller opens on a read scope.

import { tagged } from "../../../../../../wire/encode.js";
import * as blob from "../../../../../../shared/containers/oci/blob/request/request.js";
import * as manifest from "../../../../../../shared/containers/oci/manifest/request/request.js";
import * as prove from "../../../../../../shared/containers/oci/prove/request/request.js";

/**
 * What a caller asks for on a read scope answered held: three of the
 * asks of `shared.containers.oci`, each over the scope's image, the
 * cancel that ends the scope, the image's own document, and the three
 * icons the document may name.
 *
 * | tag | asks for |
 * |-----|----------|
 * | `0` | `oci_manifest`: a manifest by digest; one frame, or the empty finish |
 * | `1` | `oci_blob`: a blob by digest; pieces, each appending, then the finish |
 * | `2` | `oci_prove`: the proof over the image; one frame of 32 bytes, or the empty finish |
 * | `3` | `cancel`: carries nothing, and nothing answers on its channel; the scope's finish does |
 * | `4` | `info`: carries nothing; one frame, the image's `/.diverge/DIVERGE.md` parsed, that it ships none, an error, or that it is too large |
 * | `5` | `icon_svg`: carries nothing; one frame, the file `icons.svg` names, that there is none, an error, or that it is too large |
 * | `6` | `icon_512`: the same, of the file `icons.512` names |
 * | `7` | `icon_64`: the same, of the file `icons.64` names |
 */
export type Frame =
  | { readonly oci_manifest: manifest.Request }
  | { readonly oci_blob: blob.Request }
  | { readonly oci_prove: prove.Request }
  | "cancel"
  | "info"
  | "icon_svg"
  | "icon_512"
  | "icon_64";

const OCI_MANIFEST = 0;
const OCI_BLOB = 1;
const OCI_PROVE = 2;
const CANCEL = 3;
const INFO = 4;
const ICON_SVG = 5;
const ICON_512 = 6;
const ICON_64 = 7;

/** One byte of tag, then the ask's own JSON; the cancel, the info and the three icons are the tag alone. */
export function encode(frame: Frame): Uint8Array {
  if (frame === "cancel") {
    return Uint8Array.of(CANCEL);
  }
  if (frame === "info") {
    return Uint8Array.of(INFO);
  }
  if (frame === "icon_svg") {
    return Uint8Array.of(ICON_SVG);
  }
  if (frame === "icon_512") {
    return Uint8Array.of(ICON_512);
  }
  if (frame === "icon_64") {
    return Uint8Array.of(ICON_64);
  }
  if ("oci_manifest" in frame) {
    return tagged(OCI_MANIFEST, manifest.encode(frame.oci_manifest));
  }
  if ("oci_blob" in frame) {
    return tagged(OCI_BLOB, blob.encode(frame.oci_blob));
  }
  return tagged(OCI_PROVE, prove.encode(frame.oci_prove));
}
