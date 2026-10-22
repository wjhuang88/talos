//! Standalone contract tests for the OpenAI Responses semantic stream state machine while
//! PROVIDER-008-A wires HTTP/SSE transport and protocol selection into the provider registry.

#[path = "../src/openai_responses_reasoning.rs"]
mod openai_responses_reasoning;
#[path = "../src/openai_responses_sse.rs"]
mod openai_responses_sse;
