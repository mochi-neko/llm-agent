use crate::chat_gpt_api::memory::FiniteQueueMemory;
use crate::chat_gpt_api::specification::Model;
use crate::vector_db::database::DataBase;

#[derive(Debug)]
pub(crate) struct RpcContext {
    pub(crate) model: Model,
    pub(crate) prompt: String,
    pub(crate) context_memory: FiniteQueueMemory,
    pub(crate) long_memory: DataBase,
}
