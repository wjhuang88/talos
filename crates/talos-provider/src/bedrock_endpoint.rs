//! Regional AWS Bedrock Runtime destination for signed ConverseStream requests.

use reqwest::Url;

/// Construct the destination before looking up any AWS credentials.
pub(crate) fn converse_stream(region: &str, model: &str) -> Result<Url, &'static str> {
    if region.is_empty()
        || region.len() > 47
        || region.starts_with('-')
        || region.ends_with('-')
        || !region
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return Err("invalid Bedrock region");
    }
    if model.is_empty()
        || model.len() > 2048
        || matches!(model, "." | "..")
        || !model
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-.:_/".contains(&c))
    {
        return Err("invalid Bedrock model identifier");
    }
    let suffix = if region.starts_with("cn-") {
        "amazonaws.com.cn"
    } else {
        "amazonaws.com"
    };
    let mut url = Url::parse(&format!("https://bedrock-runtime.{region}.{suffix}"))
        .map_err(|_| "invalid Bedrock destination")?;
    url.path_segments_mut()
        .map_err(|_| "invalid Bedrock destination")?
        .extend(["model", model, "converse-stream"]);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_arn_stays_in_one_path_segment() {
        let url = converse_stream(
            "us-east-1",
            "arn:aws:bedrock:us-east-1:123:inference-profile/example",
        )
        .unwrap();
        assert_eq!(
            url.host_str(),
            Some("bedrock-runtime.us-east-1.amazonaws.com")
        );
        assert!(
            url.path()
                .ends_with("inference-profile%2Fexample/converse-stream")
        );
    }

    #[test]
    fn destination_components_cannot_replace_authority() {
        assert!(converse_stream("us-east-1@example.org", "model").is_err());
        assert!(converse_stream("us-east-1", "..").is_err());
        assert!(converse_stream("us-east-1", "model?query").is_err());
        assert_eq!(
            converse_stream("cn-north-1", "example:0")
                .unwrap()
                .host_str(),
            Some("bedrock-runtime.cn-north-1.amazonaws.com.cn")
        );
    }
}
