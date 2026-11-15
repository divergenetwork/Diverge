//! What a provider sends back on the channels a caller opens on an
//! read scope, one module per kind.
//!
//! [`oci_manifest`] and [`oci_blob`] hand over the manifests and
//! blobs the scope's image reaches; [`oci_prove`] the proof over it;
//! [`info`] the image's `/.diverge/DIVERGE.md`; [`icon_svg`],
//! [`icon_512`] and [`icon_64`] the image's three icons. The
//! cancel has no answer:
//! nothing is sent on its channel.

pub mod icon_512;
pub mod icon_64;
pub mod icon_svg;
pub mod info;
pub mod oci_blob;
pub mod oci_manifest;
pub mod oci_prove;
