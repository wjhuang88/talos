//! Exact model admission from the frozen models.dev 888c2501 source audit.
use crate::{
    AzureApiMode, AzureBodyShape, Config, ProtocolResolution, ProtocolResolutionSource,
    ProviderProtocol,
};

const RESPONSES_MODELS: &[&str] = &[
    "gpt-4.1-mini",
    "gpt-4.1-nano",
    "gpt-4.1",
    "gpt-4o-mini",
    "gpt-4o",
    "gpt-5-mini",
    "gpt-5-nano",
    "gpt-5.1",
    "gpt-5.2",
    "gpt-5.4-mini",
    "gpt-5.4-nano",
    "gpt-5.4",
    "gpt-5.5",
    "gpt-5.6-luna",
    "gpt-5.6-sol",
    "gpt-5.6-terra",
    "gpt-5",
    "gpt-6-astra",
    "gpt-6-luna",
    "gpt-6-sol",
    "gpt-6.1-sol",
    "gpt-6.1-terra",
    "gpt-chat-latest",
    "o1",
    "o3-mini",
    "o3",
    "o4-mini",
];

pub(crate) fn reviewed(config: &Config, provider: &str, model: &str) -> bool {
    if provider != "azure"
        || !RESPONSES_MODELS.contains(&model)
        || !crate::catalog_protocol::has_unoverridden_model(provider, model)
    {
        return false;
    }
    let Some(entry) = config.providers.get(provider) else {
        return false;
    };
    let Some(profile) = entry.azure.as_ref() else {
        return false;
    };
    entry.base_url.is_none()
        && matches!(profile.api, AzureApiMode::V1)
        && profile
            .resolve_model(model, entry.models.get(model))
            .is_ok()
}

impl Config {
    /// Resolve Azure shape without credential reads or resource endpoint replacement.
    /// Explicit model/provider shapes precede reviewed v1 catalog evidence, then Chat.
    pub fn resolve_azure_body_shape(
        &self,
        provider: &str,
        model: &str,
    ) -> ProtocolResolution<AzureBodyShape> {
        let entry = self.providers.get(provider);
        let explicit = entry
            .and_then(|p| p.models.get(model))
            .and_then(|m| m.azure_shape)
            .or_else(|| entry.and_then(|p| p.azure.as_ref()).and_then(|p| p.shape));
        if let Some(shape) = explicit {
            return ProtocolResolution {
                protocol: shape,
                source: ProtocolResolutionSource::Configured,
            };
        }
        if reviewed(self, provider, model)
            && self.resolve_model_protocol(provider, model).protocol
                == ProviderProtocol::AzureOpenAI
        {
            return ProtocolResolution {
                protocol: AzureBodyShape::Responses,
                source: ProtocolResolutionSource::CatalogInferred,
            };
        }
        ProtocolResolution {
            protocol: AzureBodyShape::Chat,
            source: ProtocolResolutionSource::Default,
        }
    }
}
