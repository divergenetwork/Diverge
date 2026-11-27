//! What the agent adds up to, before anything is read or written.

use std::collections::BTreeMap;

use super::{Document, PrepareError};

/// The accumulator the provider and the toolsets write into: the
/// environment, the config's inputs, the two credentials that are
/// neither, and the logins that refresh.
///
/// It holds credentials, as the agent it is read from does, and so
/// it is never printed: it derives no `Debug`.
#[derive(Default)]
pub struct Plan {
    /// The gateway's environment: the harness's plain variables —
    /// switches, endpoints, geometry — and every credential that is
    /// applied as a variable, each under the variable's own name.
    pub env: BTreeMap<String, String>,
    /// `model.provider` — the provider's own id, as its marker
    /// serializes.
    pub provider: String,
    /// `model.default` — the model, in the provider's naming.
    pub model: String,
    /// `model.base_url` — the `custom` provider's endpoint. Hermes
    /// consults `model.api_key` ONLY when the config also carries the
    /// base URL (and `model.provider` is `custom`), so the URL goes
    /// here as well as into `CUSTOM_BASE_URL`.
    pub base_url: Option<String>,
    /// `model.api_key` — the `custom` provider's key, when its
    /// arguments carry one. Hermes reads a custom key from the config
    /// and from nowhere else.
    pub api_key: Option<String>,
    /// The vertex provider's service-account document, written
    /// verbatim to the file `VERTEX_CREDENTIALS_PATH` names.
    pub vertex: Option<String>,
    /// `web.search_backend`, when a search slot names one.
    pub search_backend: Option<&'static str>,
    /// `web.extract_backend`, when the extract slot names one.
    pub extract_backend: Option<&'static str>,
    /// `browser.cloud_provider: browserbase`, when the browser's remote
    /// is Browserbase — pinned, so Hermes's auto-detect walk (Browser
    /// Use first) never enters into it.
    pub browserbase: bool,
    /// `tts.provider: elevenlabs`, when the toolset asks for it.
    pub tts_elevenlabs: bool,
    /// `image_gen.provider: fal`, when the toolset is on.
    pub image_gen: bool,
    /// `video_gen.provider: fal`, when the toolset is on.
    pub video_gen: bool,
    /// `platform_toolsets.api_server`: the toolsets exposed, by
    /// Hermes's own configurable-key names, in its order.
    pub toolsets: Vec<&'static str>,
    /// The logins that refresh, in the order the agent names them:
    /// the provider's, then spotify's.
    pub documents: Vec<Document>,
}

impl Plan {
    /// An empty plan for one model.
    pub fn new(model: String) -> Self {
        Plan {
            model,
            ..Plan::default()
        }
    }

    /// Set one environment variable: a plain one, or a credential
    /// applied as one. Setting it again to the SAME value is fine —
    /// the rule that lets `FAL_KEY` (image_gen and video_gen) and
    /// `XAI_API_KEY` (the xai provider and the x_search toolset) be
    /// named from two places; to a different one is the caller's
    /// contradiction ([`PrepareError::Contradiction`]), which names
    /// the variable and neither value.
    pub fn set(
        &mut self,
        variable: &'static str,
        value: String,
    ) -> Result<(), PrepareError> {
        match self.env.get(variable) {
            Some(existing) if *existing != value => {
                Err(PrepareError::Contradiction { variable })
            }
            Some(_) => Ok(()),
            None => {
                self.env.insert(variable.to_string(), value);
                Ok(())
            }
        }
    }
}
