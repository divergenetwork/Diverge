//! The requests over images, served: [`read`], the image itself and
//! everything read of it — its manifests, blobs, proof and DIVERGE.md
//! — from a provider or a remote daemon; [`build`], one built on a
//! provider into the daemon's Store there; [`push`], one built already
//! put there, the provider's asks for it carried to the client; and
//! [`list`], what the daemon owns in its Stores, [`get`], one image
//! of them, [`delete`], one of them taken out, or some of its
//! platforms, and [`publish`], one or more sent to a registry as one
//! image, what the provider lacks brought onto it first as a push
//! brings it.

mod bring;
pub mod build;
pub mod delete;
pub mod get;
pub mod list;
pub mod publish;
pub mod push;
pub mod read;
