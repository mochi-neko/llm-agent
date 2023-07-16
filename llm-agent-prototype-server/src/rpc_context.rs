use crate::chat_gpt_api::memory::FiniteQueueMemory;
use crate::chat_gpt_api::specification::Model;

pub(crate) struct RpcContext {
    pub(crate) model: Model,
    pub(crate) prompt: String,
    pub(crate) context_memory: FiniteQueueMemory,
}
