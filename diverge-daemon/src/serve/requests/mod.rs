//! The requests over requests, served: [`listen`], a party hearing
//! what is made of it and answering, and [`create`], a party making a
//! request of others and waiting for its answer. What both stand on
//! is [`requests`](crate::requests), the daemon's one hub; whether a
//! party is one a request is made of is [`eligible`]'s.

mod eligible;

pub use eligible::*;

pub mod create;
pub mod listen;
