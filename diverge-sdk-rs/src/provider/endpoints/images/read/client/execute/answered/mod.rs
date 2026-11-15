//! What each ask's channel decodes to: the run's manifest, blob and
//! prove answers, and an icon's, read as a caller reads them.
//!
//! One marker per ask, never constructed, so that
//! [`unary`](crate::provider::endpoints::containers::client::unary)
//! and [`ChannelStream`](crate::provider::endpoints::containers::client::ChannelStream)
//! read them. None of the three has a refusal: the OCI exchange has no error
//! vocabulary, and a provider asked for a digest the image does not
//! reach, or for a proof under a platform that names nothing,
//! finishes the channel with nothing. An icon has one: see [`Icon`].

mod icon;
mod oci_blob;
mod oci_manifest;
mod oci_prove;

pub use icon::*;
pub use oci_blob::*;
pub use oci_manifest::*;
pub use oci_prove::*;
