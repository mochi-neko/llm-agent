use crate::chat_gpt_api::specification::{CompletionResult, Options};
use anyhow::Result;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use std::env;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use super::specification::CompletionStreamingChunk;

pub(crate) async fn complete_chat(options: Options, verbose: bool) -> Result<CompletionResult> {
    if options.stream == Some(true) {
        let error = Err(anyhow::anyhow!(
            "This function is only available for stream mode"
        ));
        eprintln!("{:?}", error);
        return error;
    }

    let api_key = env::var("OPENAI_API_KEY")?;

    // HTTPS connector
    let https = HttpsConnector::new();

    // Hyper HTTP client with HTTPS support
    let client = Client::builder().build::<_, Body>(https);

    // Serialize the payload to a string
    let json_str = serde_json::to_string(&options)?;

    if verbose {
        println!("Request JSON\n{}", json_str);
    }

    // WebAPI URI
    let url = "https://api.openai.com/v1/chat/completions".parse::<hyper::Uri>()?;

    // Create HTTP POST request
    let request = Request::post(url)
        .header("Authorization", "Bearer ".to_owned() + &api_key)
        .header("Content-Type", "application/json")
        .body(Body::from(json_str))?;

    // Make the request
    let response = client.request(request).await?;

    // If the request is successful
    let status = response.status();
    if status.is_success() {
        // Read the response body
        let body_bytes = hyper::body::to_bytes(response.into_body()).await?;

        // Convert bytes to string
        let body_string = String::from_utf8(body_bytes.to_vec())?;

        if verbose {
            println!("Response JSON:\n{}", body_string);
        }

        // Deserialize the string to a struct
        let body_object = serde_json::from_str::<CompletionResult>(&body_string)?;

        Ok(body_object)
    } else {
        let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
        let body_string = String::from_utf8(body_bytes.to_vec())?;
        let error = anyhow::anyhow!(
            "HTTP request failed: {}\nResponse body: {}",
            status,
            body_string
        );

        eprintln!("{:?}", error);
        Err(error)
    }
}

pub(crate) async fn complete_chat_stream(
    tx: mpsc::UnboundedSender<Result<String>>,
    options: Options,
    verbose: bool,
) -> Result<String> {
    if options.stream == Some(true) {
        let error = Err(anyhow::anyhow!(
            "This function is only available for stream mode"
        ));
        eprintln!("{:?}", error);
        tx.send(Err(anyhow::anyhow!(
            "This function is only available for stream mode"
        )))?;
        return error;
    }

    let api_key = env::var("OPENAI_API_KEY")?;

    // HTTPS connector
    let https = HttpsConnector::new();

    // Hyper HTTP client with HTTPS support
    let client = Client::builder().build::<_, Body>(https);

    // Serialize the payload to a string
    let json_str = serde_json::to_string(&options)?;

    if verbose {
        println!("Request JSON\n{}", json_str);
    }

    // WebAPI URI
    let url = "https://api.openai.com/v1/chat/completions".parse::<hyper::Uri>()?;

    // Create HTTP POST request
    let request = Request::post(url)
        .header("Authorization", "Bearer ".to_owned() + &api_key)
        .header("Content-Type", "application/json")
        .body(Body::from(json_str))?;

    // Make the request
    match client.request(request).await {
        Err(error) => {
            eprintln!("Failed to make request: {:?}", error);
            tx.send(Err(anyhow::Error::new(error)))?;
            Err(anyhow::anyhow!("Failed to make request"))
        }
        Ok(response) => {
            // If the request is successful
            let status = response.status();
            if status.is_success() {
                let mut body = hyper::body::Body::wrap_stream(response.into_body());
                let mut total_message = "".to_string();

                while let Some(chunk) = body.next().await {
                    let chunk = chunk?;
                    let chunk_string = String::from_utf8(chunk.to_vec())?;

                    if verbose {
                        println!("Response chunk:\n{}", chunk_string);
                    }

                    // Split the chunk by newline characters and process each line
                    for line in chunk_string.split('\n') {
                        if line.is_empty() {
                            continue;
                        }
                        let result = process_chunk(tx.clone(), line.to_string(), verbose).await;
                        match result {
                            Ok(result) => {
                                total_message.push_str(&result);
                                if verbose {
                                    println!("Current total message:\n{}", total_message);
                                }
                            }
                            Err(error) => {
                                eprintln!("Failed to process chunk: {:?}", error);
                                return Err(anyhow::anyhow!("Failed to process chunk"));
                            }
                        }
                    }
                }

                if verbose {
                    println!("Result total message:\n{}", total_message);
                }

                // Finish streaming
                Ok(total_message)
            } else {
                let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
                let body_string = String::from_utf8(body_bytes.to_vec())?;
                let error = anyhow::anyhow!(
                    "HTTP request failed: {}\nResponse body: {}",
                    status.clone(),
                    body_string
                );

                eprintln!("{:?}", error);
                tx.send(Err(error))?;

                let error = anyhow::anyhow!(
                    "HTTP request failed: {}\nResponse body: {}",
                    status.clone(),
                    body_string
                );
                Err(error)
            }
        }
    }
}

async fn process_chunk(
    tx: mpsc::UnboundedSender<Result<String>>,
    line: String,
    verbose: bool,
) -> Result<String> {
    if line == "data: [DONE]" {
        if verbose {
            println!("Finish reason: DONE");
        }
        // Finished
        return Ok("".to_string());
    }

    let data = line.trim_start_matches("data: ").to_string();

    // Deserialize the string to a struct
    match serde_json::from_str::<CompletionStreamingChunk>(&data) {
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            tx.send(Err(anyhow::Error::new(e)))?;
            return Err(anyhow::anyhow!("Failed to parse JSON"));
        }
        Ok(chunk_object) => match chunk_object.choices.get(0) {
            None => return Err(anyhow::anyhow!("No choices")),
            Some(chunk_choice) => {
                if chunk_choice.finish_reason.is_some() {
                    if verbose {
                        println!("Finish reason: {:?}", chunk_choice.finish_reason);
                    }
                    // Finished
                    return Ok("".to_string());
                }
                if chunk_choice.delta.role.is_some() {
                    if verbose {
                        println!("Role: {:?}", chunk_choice.delta.role);
                    }
                    // Skip role
                    return Ok("".to_string());
                }

                match chunk_choice.delta.content.clone() {
                    None => {
                        if let Err(e) = tx.send(Err(anyhow::anyhow!("No content"))) {
                            eprintln!("Failed to send error: {:?}", e);
                        }
                        return Err(anyhow::anyhow!("No content"));
                    }
                    Some(content) => {
                        if let Err(e) = tx.send(Ok(content.clone())) {
                            eprintln!("Failed to send message: {:?}", e);
                            return Err(anyhow::anyhow!("Failed to send message"));
                        } else {
                            // Succeeded to send message
                            Ok(content)
                        }
                    }
                }
            }
        },
    }
}
