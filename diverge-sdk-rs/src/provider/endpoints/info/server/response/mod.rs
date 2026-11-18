//! What a provider sends back for an info request.
//!
//! One frame, once: the version and the capabilities — see
//! [`Frame`], and [`Capabilities`] with its
//! [`ContainersCapabilities`], [`ImagesCapabilities`], [`VolumeCapabilities`] and
//! [`MessagesCapabilities`] for what the
//! provider holds out.
//!
//! Nothing is aliased here, because nothing here is somebody else's
//! frame.

mod capabilities;
mod containers_capabilities;
mod frame;
mod images_capabilities;
mod messages_capabilities;
mod volume_capabilities;

pub use capabilities::*;
pub use containers_capabilities::*;
pub use frame::*;
pub use images_capabilities::*;
pub use messages_capabilities::*;
pub use volume_capabilities::*;
