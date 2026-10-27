//! Standalone contract tests for bridging OpenAI Responses semantic outputs into Talos invocation
//! integrity before the public LanguageModel adapter is activated.

#[path = "../src/openai_responses_bridge.rs"]
mod openai_responses_bridge;
#[path = "../src/openai_responses_reasoning.rs"]
mod openai_responses_reasoning;
#[path = "../src/openai_responses_sse.rs"]
mod openai_responses_sse;
#[path = "support/openai_responses_invocation_sender.rs"]
mod openai_sse;
