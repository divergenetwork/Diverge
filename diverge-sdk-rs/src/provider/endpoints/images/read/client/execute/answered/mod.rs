//! What each ask's channel decodes to: the run's manifest, blob and
//! prove answers, an icon's and the document's whole, read as a
//! caller reads them.
//!
//! One marker per ask, never constructed, so that
//! [`unary`](crate::provider::endpoints::containers::client::unary)
//! and [`ChannelStream`](crate::provider::endpoints::containers::client::ChannelStream)
//! read them. None of the three has a refusal: the OCI exchange has no error
//! vocabulary, and a provider asked for a digest the image does not
//! reach, or for a proof under a platform that names nothing,
//! finishes the channel with nothing. An icon and the document whole
//! have none either: every answer of theirs is the item.

mod icon;
mod info_frame;
mod oci_blob;
mod oci_manifest;
mod oci_prove;

pub use icon::*;
pub use info_frame::*;
pub use oci_blob::*;
pub use oci_manifest::*;
pub use oci_prove::*;
