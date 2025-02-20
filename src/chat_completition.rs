use crate::client::DeepinfraClient;
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_json;
use tracing::instrument;

const CHAT_COMPLETIONS_API_URL: &str = "https://api.deepinfra.com/v1/openai/chat/completions";

#[derive(Debug, Deserialize, Serialize, Builder)]
pub struct SystemMessage {
    #[builder(into)]
    content: String,
    #[builder(into)]
    name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Builder)]
pub struct UserMessage {
    #[builder(into)]
    content: String,
    #[builder(into)]
    name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Builder)]
pub struct AssistantMessage {
    #[builder(into)]
    pub content: String,
    name: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize, Serialize, Builder)]
pub struct ToolMessage {
    #[builder(into)]
    content: String,
    tool_call_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum Message {
    System(SystemMessage),
    User(UserMessage),
    Assistant(AssistantMessage),
    Tool(ToolMessage),
}

/// Represents a request for generating chat completions.
/// Includes all parameters as per the OpenAPI schema.
#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct ChatCompletionRequest {
    /// Penalizes new tokens based on their frequency in the text so far.
    /// Increases the model's likelihood to talk about new topics.
    /// Range: -2 to 2
    #[builder(default = 0.0)]
    frequency_penalty: f64,

    /// Maximum number of tokens to generate in the chat completion.
    /// Total length is limited by the model's context length.
    #[builder(default = 100000)]
    max_tokens: u32,

    /// Conversation messages including user, assistant, and system messages.
    /// Must include one system message anywhere.
    messages: Vec<Message>,

    /// Minimum probability for a token to be considered, relative to the most likely token.
    /// Must be between 0 and 1. Set to 0 to disable.
    #[builder(default = 0.0)]
    min_p: f64,

    /// Model name to use for the chat completion.
    /// Example: "meta-llama/Llama-2-70b-chat-hf"
    #[builder(default = "deepseek-ai/DeepSeek-V3".to_string())]
    model: String,

    /// Number of sequences to return.
    /// Minimum: 1, Maximum: 4
    #[builder(default = 1)]
    n: u32,

    /// Penalizes new tokens based on whether they appear in the text so far.
    /// Increases the model's likelihood to talk about new topics.
    /// Range: -2 to 2
    #[builder(default = 0.0)]
    presence_penalty: f64,

    /// Penalty for repetition. Values >1 penalize, <1 encourage repetition.
    /// Range: 0.01 to 5
    #[builder(default = 1.0)]
    repetition_penalty: f64,

    /// The format of the response. Currently, only "text" or "json_object" are supported.
    response_format: Option<ResponseFormat>,

    /// Seed for the random number generator.
    /// If not provided, a random seed is used. Determinism is not guaranteed.
    seed: Option<u64>,

    /// Up to 16 sequences where the API will stop generating further tokens.
    stop: Option<Vec<String>>,

    /// Whether to stream the output via SSE or return the full response.
    #[builder(default = false)]
    stream: bool,

    /// Sampling temperature to use, between 0 and 2.
    /// Higher values make the output more random.
    #[builder(default = 1.0)]
    temperature: f64,

    /// Controls which (if any) function is called by the model.
    /// "none" means the model will not call a function.
    /// "auto" means the model can choose to call a function or not.
    tool_choice: Option<String>,

    /// A list of tools the model may call. Currently, only functions are supported.
    tools: Option<Vec<ChatTool>>,

    /// Sample from the top_k number of tokens. 0 means off.
    #[builder(default = 0)]
    top_k: u32,

    /// Nucleus sampling parameter between 0 and 1.
    /// The model considers tokens with top_p probability mass.
    #[builder(default = 1.0)]
    top_p: f64,

    /// A unique identifier representing your end-user.
    /// Helps monitor and detect abuse. Avoid sending identifying information.
    user: Option<String>,
}

/// Represents a tool that the model may call during chat completion.
/// Currently supports functions as tools.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatTool {
    /// Type of the tool. Defaults to "function".
    #[serde(default = "default_tool_type", rename = "type")]
    type_: String,

    /// The function definition of the tool.
    function: FunctionDefinition,
}

fn default_tool_type() -> String {
    "function".to_string()
}

/// Definition of a function that can be called as a tool.
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function.
    name: String,

    /// A description of what the function does.
    description: String,

    /// Parameters for the function.
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatType {
    Text,
    JsonObject,
}

/// Specifies the format of the response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// Response type, such as "text" or "json_object".
    #[serde(default = "default_response_format_type", rename = "type")]
    pub response_type: ResponseFormatType,
}

fn default_response_format_type() -> ResponseFormatType {
    ResponseFormatType::Text
}

/// Details of a tool call made by the model.
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    /// The ID of the tool call.
    id: String,

    /// The type of the tool call. Only "function" is supported currently.
    #[serde(rename = "type")]
    type_: String,

    /// The function that the model called.
    function: FunctionCall,
}

/// Represents a function call made by the model.
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionCall {
    /// The name of the function to call.
    name: String,

    /// The function arguments in JSON format.
    /// The model may not always generate valid JSON.
    arguments: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    index: i32,
    pub message: Message,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    prompt_tokens: i32,
    total_tokens: i32,
    completion_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    id: Option<String>,
    object: Option<String>,
    created: Option<i64>,
    model: Option<String>,
    pub choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, thiserror::Error)]
pub enum ChatCompletionError {
    #[error("Request errored {0}")]
    ReqwestError(#[from] reqwest::Error),
}

type Result<T> = std::result::Result<T, ChatCompletionError>;

impl DeepinfraClient {
    /// Sends a chat completion request to DeepInfra, returning a structured response.
    ///
    /// # Usage
    /// ```
    /// let request = ChatCompletionRequestBuilder::default()
    ///     // Build your messages, model, temperature, etc.
    ///     .build();
    ///
    /// let response = client.chat_completition(request).await?;
    /// println!("Received chat response: {:?}", response);
    /// ```
    #[instrument(skip(self))]
    pub async fn chat_completition(
        &self,
        body: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let response = self
            .client
            .post(CHAT_COMPLETIONS_API_URL)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }
}
