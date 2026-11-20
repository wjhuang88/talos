//! Sign the serialized payload and send it to the regional Bedrock Runtime service.
use std::time::SystemTime;

use aws_credential_types::Credentials;
use aws_sigv4::http_request::{SignableBody, SignableRequest, SigningSettings, sign};
use aws_sigv4::sign::v4;
use reqwest::{Client, Request, header};
use talos_core::provider::{ProviderError, ProviderResult};

fn invalid(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

fn env_credentials() -> ProviderResult<Credentials> {
    let required = |key| {
        std::env::var(key)
            .ok()
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| invalid("Bedrock requires AWS environment credentials"))
    };
    let token = match std::env::var("AWS_SESSION_TOKEN") {
        Ok(v) if !v.trim().is_empty() => Some(v),
        Err(std::env::VarError::NotPresent) => None,
        _ => return Err(invalid("invalid AWS session token")),
    };
    Ok(Credentials::new(
        required("AWS_ACCESS_KEY_ID")?,
        required("AWS_SECRET_ACCESS_KEY")?,
        token,
        None,
        "TalosEnvironment",
    ))
}

pub(crate) async fn dispatch(
    region: &str,
    model: &str,
    payload: Vec<u8>,
) -> ProviderResult<reqwest::Response> {
    let url = crate::bedrock_endpoint::converse_stream(region, model).map_err(invalid)?;
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| invalid("could not initialize Bedrock transport"))?;
    let mut request = client
        .post(url)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/vnd.amazon.eventstream")
        .body(payload)
        .build()
        .map_err(|_| invalid("invalid Bedrock request"))?;
    sign_request(&mut request, region, env_credentials()?, SystemTime::now())?;
    client
        .execute(request)
        .await
        .map_err(|_| invalid("Bedrock dispatch failed"))
}

fn sign_request(
    request: &mut Request,
    region: &str,
    credentials: Credentials,
    now: SystemTime,
) -> ProviderResult<()> {
    let identity = credentials.into();
    let params = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name("bedrock")
        .settings(SigningSettings::default())
        .time(now)
        .build()
        .map_err(|_| invalid("invalid Bedrock signing parameters"))?
        .into();
    let body = request
        .body()
        .and_then(reqwest::Body::as_bytes)
        .ok_or_else(|| invalid("Bedrock requires a serialized payload"))?;
    let headers = request
        .headers()
        .iter()
        .map(|(k, v)| v.to_str().map(|value| (k.as_str(), value)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| invalid("invalid Bedrock request header"))?;
    let signable = SignableRequest::new(
        request.method().as_str(),
        request.url().as_str(),
        headers.into_iter(),
        SignableBody::Bytes(body),
    )
    .map_err(|_| invalid("invalid Bedrock signing input"))?;
    let (instructions, _) = sign(signable, &params)
        .map_err(|_| invalid("Bedrock request signing failed"))?
        .into_parts();
    for (name, value) in instructions.headers() {
        let name = header::HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| invalid("invalid signing header name"))?;
        let mut value = header::HeaderValue::from_str(value)
            .map_err(|_| invalid("invalid signing header value"))?;
        value.set_sensitive(true);
        request.headers_mut().insert(name, value);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn temporary_credentials_sign_the_final_request_sent_to_fixture() {
        let mut server = mockito::Server::new_async().await;
        let path = "/model/example%2Fprofile/converse-stream";
        let payload = r#"{"messages":[{"role":"user","content":[{"text":"hello"}]}]}"#;
        let fixture = server.mock("POST",path)
            .match_header("authorization",mockito::Matcher::Regex(
                r"^AWS4-HMAC-SHA256 Credential=TEST/19700101/us-east-1/bedrock/aws4_request, SignedHeaders=accept;content-type;host;x-amz-date;x-amz-security-token, Signature=[0-9a-f]{64}$".into()))
            .match_header("x-amz-security-token","fixture-session")
            .match_header("x-amz-date","19700101T000000Z")
            .match_body(payload)
            .with_status(200).create_async().await;
        let client = Client::new();
        let mut request = client
            .post(format!("{}{path}", server.url()))
            .header(header::ACCEPT, "application/vnd.amazon.eventstream")
            .header(header::CONTENT_TYPE, "application/json")
            .body(payload)
            .build()
            .unwrap();
        let credentials = Credentials::new(
            "TEST",
            "fixture-secret",
            Some("fixture-session".into()),
            None,
            "fixture",
        );
        sign_request(
            &mut request,
            "us-east-1",
            credentials,
            SystemTime::UNIX_EPOCH,
        )
        .unwrap();
        assert!(request.headers()[header::AUTHORIZATION].is_sensitive());
        assert!(request.headers()["x-amz-security-token"].is_sensitive());
        assert!(client.execute(request).await.unwrap().status().is_success());
        fixture.assert_async().await;
    }
}
