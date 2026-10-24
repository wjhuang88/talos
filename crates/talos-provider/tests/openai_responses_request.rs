//! Standalone contract tests for the OpenAI Responses request shaper while PROVIDER-008-A wires the
//! adapter into the public provider registry. Keeping the source module path-included here lets the
//! request contract compile and run before protocol selection is activated.

#[path = "../src/image_io.rs"]
mod image_io;
#[path = "../src/openai_responses_reasoning.rs"]
mod openai_responses_reasoning;
#[path = "../src/openai_responses_request.rs"]
mod openai_responses_request;
