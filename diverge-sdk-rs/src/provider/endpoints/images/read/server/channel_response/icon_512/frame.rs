//! What the 512 by 512 PNG icon's channel answers.

/// The scope's image's 512 by 512 PNG icon — the file its document's
/// `icons.512` names — as its bytes; that the image ships none;
/// an error; or that the file is too large. See
/// [`icon::response::Frame`](crate::shared::containers::icon::response::Frame).
///
/// An alias rather than a re-export because this module is real: the
/// path says this scope's answer lives here, and it does, rather than
/// naming somewhere else and hoping a reader follows. What the alias
/// points at is right there in the signature.
pub type Frame = crate::shared::containers::icon::response::Frame;

/// An answer on this channel that could not be read.
pub type FrameError = crate::shared::containers::icon::response::FrameError;
