//! Standalone contract tests for bridging OpenAI Responses semantic outputs into Talos invocation
//! integrity before the public LanguageModel adapter is activated.

#[path = "../src/browser_text_arguments.rs"]
mod browser_text_arguments;
#[path = "../src/stream_utf8.rs"]
mod stream_utf8;
#[path = "../src/anthropic_stream.rs"]
mod anthropic_stream;
#[path = "../src/openai_sse.rs"]
mod openai_sse;
#[path = "../src/openai_responses_reasoning.rs"]
mod openai_responses_reasoning;
#[path = "../src/openai_responses_sse.rs"]
mod openai_responses_sse;
#[path = "../src/openai_responses_bridge.rs"]
mod openai_responses_bridge;
