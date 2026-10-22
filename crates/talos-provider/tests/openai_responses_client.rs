//! Standalone contract tests for the staged OpenAI Responses HTTP client before runtime activation.

#[path = "../src/image_io.rs"]
mod image_io;
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
#[path = "../src/retry.rs"]
mod retry;
#[path = "../src/stream_utf8.rs"]
mod stream_utf8;
