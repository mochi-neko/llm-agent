use crate::chat_gpt_api::specification::{CompletionResult, Options};
use anyhow::Result;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use std::env;

pub(crate) async fn complete_chat(options: Options) -> Result<CompletionResult> {
    let span = tracing::span!(tracing::Level::DEBUG, "complete chat");
    let _enter = span.enter();

    if options.stream == Some(true) {
        let error = Err(anyhow::anyhow!(
            "This function is only available for stream mode"
        ));
        tracing::error!("{:?}", error);
        return error;
    }

    let api_key = env::var("OPENAI_API_KEY").map_err(|error| {
        tracing::error!("Failed to get OPENAI_API_KEY: {:?}", error);
        error
    })?;

    // HTTPS connector
    let https = HttpsConnector::new();

    // Hyper HTTP client with HTTPS support
    let client = Client::builder().build::<_, Body>(https);

    // Serialize the payload to a string
    let json_str = serde_json::to_string(&options).map_err(|error| {
        tracing::error!("Failed to serialize JSON: {:?}", error);
        error
    })?;

    tracing::info!("Request JSON:\n{}", json_str);

    // WebAPI URI
    let url = "https://api.openai.com/v1/chat/completions"
        .parse::<hyper::Uri>()
        .map_err(|error| {
            tracing::error!("Failed to parse URI: {:?}", error);
            error
        })?;

    // Create HTTP POST request
    let request = Request::post(url)
        .header("Authorization", "Bearer ".to_owned() + &api_key)
        .header("Content-Type", "application/json")
        .body(Body::from(json_str))
        .map_err(|error| {
            tracing::error!("Failed to create request: {:?}", error);
            error
        })?;

    // Make the request
    let response = client.request(request).await.map_err(|error| {
        tracing::error!("Failed to make request: {:?}", error);
        error
    })?;

    // If the request is successful
    let status = response.status();
    if status.is_success() {
        // Read the response body
        let body_bytes = hyper::body::to_bytes(response.into_body())
            .await
            .map_err(|error| {
                tracing::error!("Failed to read response body: {:?}", error);
                error
            })?;

        // Convert bytes to string
        let body_string = String::from_utf8(body_bytes.to_vec()).map_err(|error| {
            tracing::error!("Failed to convert bytes to string: {:?}", error);
            error
        })?;

        tracing::info!("Response JSON:\n{}", body_string);

        // Deserialize the string to a struct
        let body_object =
            serde_json::from_str::<CompletionResult>(&body_string).map_err(|error| {
                tracing::error!("Failed to deserialize JSON: {:?}", error);
                error
            })?;

        Ok(body_object)
    } else {
        let body_bytes = hyper::body::to_bytes(response.into_body())
            .await
            .map_err(|error| {
                tracing::error!("Failed to read error response body: {:?}", error);
                error
            })?;

        let body_string = String::from_utf8(body_bytes.to_vec()).map_err(|error| {
            tracing::error!("Failed to convert error bytes to string: {:?}", error);
            error
        })?;

        let error = anyhow::anyhow!(
            "HTTP request failed: {}\nResponse body: {}",
            status,
            body_string
        );

        tracing::error!("{:?}", error);
        Err(error)
    }
}
