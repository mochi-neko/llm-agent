use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum Model {
    Gpt35Turbo,
    Gpt35Turbo0613,
    Gpt35Turbo16k,
    Gpt35Turbo16k0613,
    Gpt4,
    Gpt40613,
    Gpt432k,
    Gpt432k0613,
}

impl Model {
    pub(crate) fn parse_to_string(&self) -> Result<String> {
        match self {
            | Model::Gpt35Turbo => Ok("gpt-3.5-turbo".to_string()),
            | Model::Gpt35Turbo0613 => Ok("gpt-3.5-turbo-0613".to_string()),
            | Model::Gpt35Turbo16k => Ok("gpt-3.5-turbo-16k".to_string()),
            | Model::Gpt35Turbo16k0613 => {
                Ok("gpt-3.5-turbo-16k-0613".to_string())
            },
            | Model::Gpt4 => Ok("gpt-4".to_string()),
            | Model::Gpt40613 => Ok("gpt-4-0613".to_string()),
            | Model::Gpt432k => Ok("gpt-4-32k".to_string()),
            | Model::Gpt432k0613 => Ok("gpt-4-32k-0613".to_string()),
        }
    }

    // pub(crate) fn parse_to_model(input: &str) -> Result<Model> {
    //     match input {
    //         "gpt-3.5-turbo" => Ok(Model::Gpt35Turbo),
    //         "gpt-3.5-turbo-0613" => Ok(Model::Gpt35Turbo0613),
    //         "gpt-3.5-turbo-16k" => Ok(Model::Gpt35Turbo16k),
    //         "gpt-3.5-turbo-16k-0613" => Ok(Model::Gpt35Turbo16k0613),
    //         "gpt-4" => Ok(Model::Gpt4),
    //         "gpt-4-0613" => Ok(Model::Gpt40613),
    //         "gpt-4-32k" => Ok(Model::Gpt432k),
    //         "gpt-4-32k-0613" => Ok(Model::Gpt432k0613),
    //         _ => Err(anyhow!("Invalid model")),
    //     }
    // }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum Role {
    System,
    Assistant,
    User,
    Function,
}

impl Role {
    pub(crate) fn parse_to_string(&self) -> Result<String> {
        match self {
            | Role::System => Ok("system".to_string()),
            | Role::Assistant => Ok("assistant".to_string()),
            | Role::User => Ok("user".to_string()),
            | Role::Function => Ok("function".to_string()),
        }
    }

    // pub(crate) fn parse_to_role(input: &str) -> Result<Role> {
    //     match input {
    //         "system" => Ok(Role::System),
    //         "assistant" => Ok(Role::Assistant),
    //         "user" => Ok(Role::User),
    //         "function" => Ok(Role::Function),
    //         _ => Err(anyhow!("Invalid role")),
    //     }
    // }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Options {
    pub(crate) model: String,
    pub(crate) messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) functions: Option<Vec<Function>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) function_call: Option<FunctionCallingSpecification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) n: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) logit_bias: Option<HashMap<String, f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) user: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Function {
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    pub(crate) parameters: serde_json::Map<String, serde_json::Value>,
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.parameters.clone(),
        }
    }
}

impl Function {
    pub(crate) fn new(
        name: String,
        description: Option<String>,
        parameters_schema: String,
    ) -> Function {
        Function {
            name,
            description,
            parameters: serde_json::from_str(&parameters_schema).unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) enum FunctionCallingSpecification {
    Auto,
    None,
    Name(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Message {
    pub(crate) role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) function_call: Option<FunctionCall>,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            role: self.role.clone(),
            content: self.content.clone(),
            name: self.name.clone(),
            function_call: self.function_call.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CompletionResult {
    pub(crate) id: String,
    pub(crate) object: String,
    pub(crate) created: u64,
    pub(crate) model: String,
    pub(crate) choices: Vec<Choice>,
    pub(crate) usage: Usage,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Choice {
    pub(crate) index: u64,
    pub(crate) message: Message,
    pub(crate) finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FunctionCall {
    pub(crate) name: String,
    pub(crate) arguments: String,
}

impl Clone for FunctionCall {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            arguments: self.arguments.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Usage {
    pub(crate) prompt_tokens: u64,
    pub(crate) completion_tokens: u64,
    pub(crate) total_tokens: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CompletionStreamingChunk {
    pub(crate) id: String,
    pub(crate) object: String,
    pub(crate) created: u64,
    pub(crate) model: String,
    pub(crate) choices: Vec<ChoiceChunk>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChoiceChunk {
    pub(crate) delta: Delta,
    pub(crate) index: u64,
    pub(crate) finish_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content: Option<String>,
}
