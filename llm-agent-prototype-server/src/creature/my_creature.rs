pub(crate) mod creature_rpc {
    tonic::include_proto!("creature");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("creature_descriptor");
}

use crate::chat_gpt_api::memory::Memory;
use crate::chat_gpt_api::specification::{
    Function, FunctionCallingSpecification, Message, Options, Role,
};
use crate::rpc_context::RpcContext;
use crate::vector_db::database::MetaData;
use creature_rpc::creature_server::Creature;
use creature_rpc::{Cry, Emotion, Motion};
use futures::stream::StreamExt;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, MutexGuard};
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Response, Status};

#[derive(Debug)]
pub struct MyCreature {
    pub(crate) context: Arc<Mutex<RpcContext>>,
}

#[derive(serde::Deserialize, Debug)]
struct StateJson {
    emotion: String,
    motion: String,
    cry: String,
    friendliness: f64,
}

#[tonic::async_trait]
impl Creature for MyCreature {
    type TalkStream = Pin<
        Box<
            dyn Stream<Item = Result<creature_rpc::State, Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    // grpcurl -plaintext -d '{ "message": "おはよう!" }' localhost:8000 creature.Creature/Talk
    #[tracing::instrument(
        name = "creature.talk",
        err,
        skip(self, request)
    )]
    async fn talk(
        &self,
        request: tonic::Request<tonic::Streaming<creature_rpc::Talking>>,
    ) -> std::result::Result<tonic::Response<Self::TalkStream>, tonic::Status>
    {
        tracing::info!("Request talk: {:?}", request);

        let mut stream = request.into_inner();

        let (tx, rx) = mpsc::channel(100);

        let context = self.context.clone();

        tokio::spawn(async move {
            while let Some(request) = stream.next().await {
                let request = match request {
                    | Ok(req) => req,
                    | Err(e) => {
                        tracing::error!("Failed to receive request: {:?}", e);
                        let _ = tx.send(Err(e)).await;
                        break;
                    },
                };
                let context = context.lock().await;
                let response = match react(context, request).await {
                    | Ok(resp) => resp,
                    | Err(e) => {
                        tracing::error!("Failed to react: {:?}", e);
                        let _ = tx.send(Err(e)).await;
                        break;
                    },
                };
                if tx
                    .send(Ok(response))
                    .await
                    .is_err()
                {
                    tracing::error!("Failed to send response");
                    break;
                }
            }
        });

        let outgoing = ReceiverStream::new(rx);

        tracing::info!(
            "Response talk streaming: {:?}",
            outgoing
        );

        Ok(Response::new(
            Box::pin(outgoing) as Self::TalkStream
        ))
    }
}

fn build_messages(
    prompt: String,
    context: Vec<Message>,
) -> Vec<Message> {
    let mut messages = Vec::new();

    messages.push(Message {
        role: Role::System
            .parse_to_string()
            .unwrap(),
        content: Some(prompt),
        name: None,
        function_call: None,
    });

    for message in context {
        messages.push(message);
    }

    messages
}

#[tracing::instrument(
    name = "creature.talk_react",
    err,
    skip(context, talking)
)]
async fn react(
    mut context: MutexGuard<'_, RpcContext>,
    talking: creature_rpc::Talking,
) -> Result<creature_rpc::State, Status> {
    tracing::info!(
        "Request react to talking: {:?}",
        talking
    );

    context
        .context_memory
        .add(Message {
            role: Role::User
                .parse_to_string()
                .unwrap(),
            content: Some(talking.message.clone()),
            name: None,
            function_call: None,
        });

    context
        .long_memory
        .upsert(
            talking.message,
            MetaData::new("Mochineko".to_string()), // TODO: Set author from parameters.
        )
        .await
        .map_err(|error| {
            tracing::error!(
                "Failed to upsert message to long memory: {:?}",
                error
            );
            Status::new(
                tonic::Code::Internal,
                "Failed to upsert to long memory".to_string(),
            )
        })?;

    let context_memory = context.context_memory.get();
    let messages = build_messages(
        context.prompt.clone(),
        context_memory.clone(),
    );
    let functions = vec![Function::new(
        "reaction_generator".to_string(),
        Some("Generate your reaction as character of creature from conversations.".to_string()),
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
                        "MOTION_NO",
                        "MOTION_JUMP",
                        "MOTION_DIE",
                        "MOTION_RUN",
                        "MOTION_WALK",
                        "MOTION_FLYING",
                        "MOTION_ATTACK",
                        "MOTION_EATING"
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
                },
                "friendliness" : {
                    "type": "number",
                    "description": "Friendliness of creature that changes slowly by user interaction.",
                    "minimum": -1,
                    "maximum": 1
                }
            },
            "required": [
                "emotion",
                "motion",
                "cry",
                "friendliness"
            ]
        }"#
        .to_string(),
    )];

    let options: Options = Options {
        model: context
            .model
            .parse_to_string()
            .unwrap(),
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

    tracing::info!(
        "Request complete chat to react with options: {:?}",
        options
    );

    match crate::chat_gpt_api::client::complete_chat(options).await {
        | Err(error) => {
            tracing::error!(
                "Failed to complete chat to react: {:?}",
                error
            );
            Err(
                crate::error_mapping::map_anyhow_error_to_grpc_status(
                    anyhow::anyhow!(
                        "Failed to complete chat to react: {:?}",
                        error
                    ),
                ),
            )
        },
        | Ok(response) => match response.choices.get(0) {
            | None => {
                tracing::error!("No choices in response");
                Err(Status::new(
                    tonic::Code::Internal,
                    "No choices in response".to_string(),
                ))
            },
            | Some(choice) => match &choice.message.function_call {
                | None => {
                    tracing::error!("No function calling in response");
                    Err(Status::new(
                        tonic::Code::Internal,
                        "No function calling in response".to_string(),
                    ))
                },
                // Success
                | Some(function_call) => {
                    context
                        .context_memory
                        .add(Message {
                            role: Role::Assistant
                                .parse_to_string()
                                .unwrap(),
                            content: Some("".to_string()), // NOTE: Must be set some.
                            name: choice.message.name.clone(),
                            function_call: choice
                                .message
                                .function_call
                                .clone(),
                        });

                    let reaction = serde_json::from_str::<StateJson>(
                        &function_call.arguments,
                    )
                    .map_err(|error| {
                        tracing::error!(
                            "Failed to parse function calling arguments: {:?}",
                            error
                        );
                        Status::new(
                            tonic::Code::Internal,
                            "Failed to parse function calling arguments"
                                .to_string(),
                        )
                    })?;

                    let emotion = Emotion::from_str_name(&reaction.emotion)
                        .ok_or_else(|| {
                            tracing::error!(
                                "Invalid emotion: {:?}",
                                reaction.emotion
                            );
                            Status::new(
                                tonic::Code::Internal,
                                "Failed to parse emotion".to_string(),
                            )
                        })?;

                    let motion = Motion::from_str_name(&reaction.motion)
                        .ok_or_else(|| {
                            tracing::error!(
                                "Invalid motion: {:?}",
                                reaction.motion
                            );
                            Status::new(
                                tonic::Code::Internal,
                                "Failed to parse motion".to_string(),
                            )
                        })?;

                    let cry =
                        Cry::from_str_name(&reaction.cry).ok_or_else(|| {
                            tracing::error!("Invalid cry: {:?}", reaction.cry);
                            Status::new(
                                tonic::Code::Internal,
                                "Failed to parse cry".to_string(),
                            )
                        })?;

                    let state = creature_rpc::State {
                        emotion: emotion as i32,
                        motion: motion as i32,
                        cry: cry as i32,
                    };

                    tracing::info!("Succeeded to react: {:?}", state);

                    Ok(state)
                },
            },
        },
    }
}
