//! Confirmed native thinking controls; never guess capabilities from a model prefix.
//! Source: https://ai.google.dev/gemini-api/docs/generate-content/thinking (2026-11-02).

use serde_json::{Value, json};
use talos_config::{ReasoningEffort, ReasoningOptions};
use talos_core::provider::{ProviderError, ProviderResult};

fn invalid(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

pub(crate) fn thinking_config(
    model: &str,
    options: Option<&ReasoningOptions>,
) -> ProviderResult<Option<Value>> {
    let Some(options) = options else {
        return Ok(None);
    };
    if !options.replay {
        return Err(invalid(
            "Gemini requires native signature replay; replay=false is unsupported",
        ));
    }
    let model = model.strip_prefix("models/").unwrap_or(model);
    let mut config = json!({"includeThoughts":true});
    match model {
        "gemini-2.5-pro" | "gemini-2.5-flash" | "gemini-2.5-flash-lite" => {
            if options.effort.is_some() {
                return Err(invalid(
                    "Gemini 2.5 requires a thinking budget, not reasoning effort",
                ));
            }
            if let Some(budget) = options.budget_tokens {
                let valid = match model {
                    "gemini-2.5-pro" => (128..=32768).contains(&budget),
                    "gemini-2.5-flash" => budget <= 24576,
                    _ => budget == 0 || (512..=24576).contains(&budget),
                };
                if !valid {
                    return Err(invalid(
                        "Gemini thinking budget is outside the confirmed model range",
                    ));
                }
                config["thinkingBudget"] = json!(budget);
            }
        }
        "gemini-3.8-flash" | "gemini-4-argon" => {
            if options.budget_tokens.is_some() {
                return Err(invalid(
                    "this Gemini model requires reasoning effort, not a thinking budget",
                ));
            }
            if let Some(effort) = &options.effort {
                config["thinkingLevel"] = json!(match effort {
                    ReasoningEffort::Low => "low",
                    ReasoningEffort::Medium => "medium",
                    ReasoningEffort::High => "high",
                });
            } else if model == "gemini-4-argon" {
                config["thinkingLevel"] = json!("medium");
            }
        }
        _ => {
            return Err(invalid(
                "Gemini reasoning controls are not confirmed for this model",
            ));
        }
    }
    Ok(Some(config))
}

/// Independent low-latency decision policy; never reuse conversation options.
pub(crate) fn decision_config(model: &str) -> Option<Value> {
    let model = model.strip_prefix("models/").unwrap_or(model);
    match model {
        "gemini-2.5-flash" | "gemini-2.5-flash-lite" => {
            Some(json!({"includeThoughts":false,"thinkingBudget":0}))
        }
        "gemini-2.5-pro" => Some(json!({"includeThoughts":false,"thinkingBudget":128})),
        "gemini-3.8-flash" | "gemini-4-argon" => {
            Some(json!({"includeThoughts":false,"thinkingLevel":"low"}))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmed_budget_boundaries_do_not_admit_aliases_or_wrong_controls() {
        for (model, allowed, denied) in [
            ("gemini-2.5-pro", vec![128, 32768], vec![0, 127, 32769]),
            ("gemini-2.5-flash", vec![0, 24576], vec![24577]),
            (
                "gemini-2.5-flash-lite",
                vec![0, 512, 24576],
                vec![1, 511, 24577],
            ),
        ] {
            for budget in allowed {
                let options = ReasoningOptions {
                    budget_tokens: Some(budget),
                    ..Default::default()
                };
                let wire = thinking_config(model, Some(&options))
                    .expect("admit")
                    .expect("config");
                assert_eq!(
                    wire,
                    json!({"includeThoughts":true,"thinkingBudget":budget})
                );
            }
            for budget in denied {
                assert!(
                    thinking_config(
                        model,
                        Some(&ReasoningOptions {
                            budget_tokens: Some(budget),
                            ..Default::default()
                        })
                    )
                    .is_err()
                );
            }
            assert!(
                thinking_config(
                    model,
                    Some(&ReasoningOptions {
                        effort: Some(ReasoningEffort::Low),
                        ..Default::default()
                    })
                )
                .is_err()
            );
        }
        for model in [
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "gemini-2.5-flash-lite",
            "gemini-3.8-flash",
            "gemini-4-argon",
        ] {
            assert!(
                thinking_config(
                    model,
                    Some(&ReasoningOptions {
                        effort: Some(ReasoningEffort::Low),
                        budget_tokens: Some(8192),
                        ..Default::default()
                    })
                )
                .is_err(),
                "mixed controls must not reach the wire"
            );
        }
        for model in [
            "gateway/gemini-2.5-flash",
            "gemini-2.5-pro-preview-06-05",
            "unknown",
        ] {
            assert!(thinking_config(model, Some(&Default::default())).is_err());
            assert!(thinking_config(model, None).expect("no controls").is_none());
        }
    }

    #[test]
    fn levels_and_summary_defaults_preserve_native_replay_boundary() {
        for model in ["gemini-3.8-flash", "gemini-4-argon"] {
            for (effort, expected) in [
                (ReasoningEffort::Low, "low"),
                (ReasoningEffort::Medium, "medium"),
                (ReasoningEffort::High, "high"),
            ] {
                assert_eq!(
                    thinking_config(
                        model,
                        Some(&ReasoningOptions {
                            effort: Some(effort),
                            ..Default::default()
                        })
                    )
                    .expect("level"),
                    Some(json!({"includeThoughts":true,"thinkingLevel":expected}))
                );
            }
            assert!(
                thinking_config(
                    model,
                    Some(&ReasoningOptions {
                        budget_tokens: Some(1024),
                        ..Default::default()
                    })
                )
                .is_err()
            );
        }
        assert_eq!(
            thinking_config("models/gemini-2.5-pro", Some(&Default::default())).expect("dynamic"),
            Some(json!({"includeThoughts":true}))
        );
        assert_eq!(
            thinking_config("gemini-4-argon", Some(&Default::default()))
                .expect("documented default"),
            Some(json!({"includeThoughts":true,"thinkingLevel":"medium"}))
        );
        assert!(
            thinking_config(
                "gemini-2.5-flash",
                Some(&ReasoningOptions {
                    replay: false,
                    ..Default::default()
                })
            )
            .is_err()
        );
    }
}
