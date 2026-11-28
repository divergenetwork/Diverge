//! What a client's channel request frame carries for an API
//! request: its input, as [`api`](crate::shared::containers::api) states it.

pub use crate::shared::containers::api::input::{Input as Frame, InputError as FrameError};
