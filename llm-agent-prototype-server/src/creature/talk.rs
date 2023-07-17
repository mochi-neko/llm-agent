pub(crate) mod creature_rpc {
    tonic::include_proto!("creature");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("creature_descriptor");
}

use crate::chat_gpt_api::client::complete_chat;
use crate::chat_gpt_api::memory::Memory;
use crate::chat_gpt_api::specification::{
    Function, FunctionCallingSpecification, Message, Options, Role,
};
use crate::error_conversion::map_anyhow_error_to_grpc_status;
use crate::rpc_context::RpcContext;
use creature_rpc::creature_server::Creature;
use creature_rpc::{Cry, Emotion, Motion};
use futures::stream::StreamExt;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tokio::sync::mpsc;

pub struct MyCreature {
    pub(crate) state: Arc<Mutex<RpcContext>>,
}

#[derive(serde::Deserialize, Debug)]
struct StateJson {
    emotion: String,
    motion: String,
    cry: String,
}

#[tonic::async_trait]
impl Creature for MyCreature {
    type TalkStream =
        Pin<Box<dyn Stream<Item = Result<creature_rpc::State, Status>> + Send + Sync + 'static>>;

    // grpcurl -plaintext -d '{ "message": "おはよう!" }' localhost:8000 speak.Speak/SpeakTo
    async fn talk(
        &self,
        request: tonic::Request<tonic::Streaming<creature_rpc::Talking>>,
    ) -> std::result::Result<tonic::Response<Self::TalkStream>, tonic::Status> {
        let state = self.state.clone();
        let mut stream = request.into_inner();

        let (tx, rx) = mpsc::channel(100); // adjust the size as needed

        tokio::spawn(async move {
            while let Some(request) = stream.next().await {
                let request = match request {
                    Ok(req) => req,
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                };
                let response = match self.react(request).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                };
                if tx.send(Ok(response)).await.is_err() {
                    break;
                }
            }
        });

        let outgoing = rx.map(|res| res);
        Ok(Response::new(Box::pin(outgoing) as Self::TalkStream))
    }
}

impl MyCreature {
    async fn react(&self, talking: creature_rpc::Talking) -> Result<creature_rpc::State, Status> {
        let mut state = self.state.lock().await;

        state.context_memory.add(Message {
            role: Role::User.parse_to_string().unwrap(),
            content: Some(talking.message),
            name: None,
            function_call: None,
        });

        let context = state.context_memory.get();
        let messages = build_messages(state.prompt.clone(), context.clone());
        let functions = vec![Function::new(
            "reaction_generator".to_string(),
            Some("Generate reaction of AI character like Pokemon from conversations.".to_string()),
            r#"{
                "type": "object",
                "properties": {
                    "emotion": {
                        "type": "string",
                        "enum": [
                            "EMOTION_NEUTRAL",
                            "EMOTION_HAPPY",
                            "EMOTION_SAD",
                            "EMOTION_ANGRY",
                            "EMOTION_FEARFUL",
                            "EMOTION_DISGUSTED",
                            "EMOTION_SURPRISED"
                        ]
                    },
                    "motion": {
                        "type": "string",
                        "enum": [
                            "MOTION_NEUTRAL",
                            "MOTION_HAPPY",
                            "MOTION_SAD",
                            "MOTION_ANGRY",
                            "MOTION_FEARFUL",
                            "MOTION_DISGUSTED",
                            "MOTION_SURPRISED",
                            "MOTION_DANCE",
                            "MOTION_FLOAT",
                            "MOTION_SLEEP"
                        ]
                    },
                    "cry": {
                        "type": "string",
                        "enum": [
                            "CRY_NONE",
                            "CRY_HAPPY",
                            "CRY_SAD",
                            "CRY_ANGRY",
                            "CRY_FEARFUL",
                            "CRY_DISGUSTED",
                            "CRY_SURPRISED",
                            "CRY_SPOILED",
                            "CRY_CRY"
                        ]
                    }
                },
                "required": [
                    "emotion",
                    "motion",
                    "cry"
                ]
            }"#
            .to_string(),
        )];

        let options: Options = Options {
            model: state.model.parse_to_string().unwrap(),
            messages,
            functions: Some(functions),
            function_call: Some(FunctionCallingSpecification::Name(
                "reaction_generator".to_string(),
            )),
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        };

        match complete_chat(options, true).await {
            Err(error) => {
                let error = anyhow::anyhow!("Error in speak to: {:?}", error);
                Err(map_anyhow_error_to_grpc_status(error))
            }
            Ok(response) => match response.choices.get(0) {
                None => Err(Status::new(
                    tonic::Code::Internal,
                    "No choices in response".to_string(),
                )),
                Some(choice) => match &choice.message.function_call {
                    None => Err(Status::new(
                        tonic::Code::Internal,
                        "No function calling in response".to_string(),
                    )),
                    // Success
                    Some(function_call) => {
                        state.context_memory.add(Message {
                            role: Role::Function.parse_to_string().unwrap(),
                            content: None,
                            name: None,
                            function_call: Some(function_call.clone()),
                        });

                        let speak_reaction =
                            serde_json::from_str::<StateJson>(&function_call.arguments).unwrap();

                        Ok(creature_rpc::State {
                            emotion: Emotion::from_str_name(&speak_reaction.emotion).unwrap()
                                as i32,
                            motion: Motion::from_str_name(&speak_reaction.motion).unwrap() as i32,
                            cry: Cry::from_str_name(&speak_reaction.cry).unwrap() as i32,
                        })
                    }
                },
            },
        }
    }
}

fn build_messages(prompt: String, context: Vec<Message>) -> Vec<Message> {
    let mut messages = Vec::new();

    messages.push(Message {
        role: Role::System.parse_to_string().unwrap(),
        content: Some(prompt),
        name: None,
        function_call: None,
    });

    for message in context {
        messages.push(message);
    }

    messages
}
