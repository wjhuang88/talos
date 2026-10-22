//! Standalone contract tests for the OpenAI Responses public provider seam before crate export.

#[path = "../src/image_io.rs"]
mod image_io;
#[path = "../src/openai_responses.rs"]
mod openai_responses;
#[path = "../src/openai_responses_bridge.rs"]
mod openai_responses_bridge;
#[path = "../src/openai_responses_client.rs"]
mod openai_responses_client;
#[path = "../src/openai_responses_reasoning.rs"]
mod openai_responses_reasoning;
#[path = "../src/openai_responses_request.rs"]
mod openai_responses_request;
#[path = "../src/openai_responses_sse.rs"]
mod openai_responses_sse;
#[path = "../src/openai_responses_transport.rs"]
mod openai_responses_transport;
#[path = "support/openai_responses_invocation_sender.rs"]
mod openai_sse;
#[path = "../src/retry.rs"]
mod retry;
#[path = "../src/stream_utf8.rs"]
mod stream_utf8;

fn validate_decision_messages(
    messages: &[talos_core::message::Message],
    limits: talos_core::provider::DecisionRequestLimits,
) -> talos_core::provider::ProviderResult<()> {
    if limits.max_output_tokens == 0
        || messages.iter().any(|message| {
            !matches!(
                message,
                talos_core::message::Message::System { .. }
                    | talos_core::message::Message::User { .. }
            )
        })
    {
        return Err(talos_core::provider::ProviderError::InvalidResponse(
            "invalid isolated decision request".into(),
        ));
    }
    Ok(())
}
