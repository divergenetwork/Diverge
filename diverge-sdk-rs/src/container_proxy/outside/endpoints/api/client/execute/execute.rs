//! Opening the scope.

use super::super::request;
use super::{ExecuteError, ExecuteHandle, ExecuteStream};
use crate::container_proxy::outside::endpoints::frontend::client::execute::open;
use crate::shared::containers::api;
use crate::wire::client::handle::Handle;
use crate::wire::encode::{Encode, Writer};

/// Open an API request with `head`: the answer as a stream, and the
/// handle that writes its body and cancels it. Dropping the stream
/// abandons it, which drops the request too.
pub async fn execute(handle: &Handle, head: api::request::Head) -> Result<(ExecuteStream, ExecuteHandle), ExecuteError> {
    let mut payload = Vec::new();
    request::Frame(head)
        .encode(&mut Writer::new(&mut payload))
        .map_err(ExecuteError::Request)?;
    open(handle, &payload).await
}
