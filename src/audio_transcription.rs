use crate::client::DeepinfraClient;
use bon::Builder;
use reqwest::multipart;
use serde::Deserialize;
use std::path::Path;
use tracing::instrument;

const AUDIO_TRANSCRIPTION_API_URL: &str =
    "https://api.deepinfra.com/v1/openai/audio/transcriptions";

#[derive(Debug, Deserialize)]
pub struct AudioTranscriptionResponse {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorDetail {
    loc: Vec<String>,
    msg: String,
    #[serde(rename = "type")]
    error_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ErrorResponse {
    Simple { detail: String },
    Detailed { detail: Vec<ErrorDetail> },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AudioTranscriptionApiResponse {
    TranscriptionResponse(AudioTranscriptionResponse),
    ErrorResponse(ErrorResponse),
}

#[derive(Debug, thiserror::Error)]
pub enum AudioTranscriptionError {
    #[error("Request error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("File not found: {0}")]
    FileNotFoundError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Error response: {0}")]
    ErrorResponse(String),
}

#[derive(Debug)]
pub enum FileSource {
    Filepath(Box<Path>),
    Bytes { buffer: Vec<u8>, file_name: String },
}

#[derive(Builder)]
/// Represents a request to transcribe an audio file.
///
/// # Fields
/// - `language`: Optional language of the input audio (ISO-639-1 format).
/// - `model`: The transcription model to use (default: "openai/whisper-large-v3-turbo").
/// - `prompt`: Optional prompt to guide the transcription output.
/// - `response_format`: The desired format of the transcription response (e.g., "json", "text").
/// - `source`: The audio source; can be either a file path or a byte buffer.
/// - `temperature`: Optional sampling temperature (between 0 and 1).
/// - `timestamp_granularities`: Optional list specifying timestamp granularities.
pub struct AudioTranscriptionRequest {
    /// Optional language of the input audio in ISO-639-1 format.
    #[builder(into)]
    language: Option<String>,
    /// The transcription model to use (default: "openai/whisper-large-v3-turbo").
    #[builder(default = "openai/whisper-large-v3-turbo".to_string())]
    model: String,
    /// Optional prompt to guide the transcription style.
    #[builder(into)]
    prompt: Option<String>,
    /// The desired response format (default: "json").
    #[builder(default = "json".to_string())]
    response_format: String,
    /// The audio source: either a file path or a buffer with file name.
    #[builder(into)]
    source: FileSource,
    /// Optional temperature controlling sampling; must be between 0 and 1.
    #[builder(into)]
    temperature: Option<f32>,
    /// Optional timestamp granularities for transcription.
    #[builder(into)]
    timestamp_granularities: Option<Vec<String>>,
}

impl DeepinfraClient {
    /// Transcribes an audio file using the Deepinfra API.
    ///
    /// This function builds and sends a multipart/form request containing the audio file and additional
    /// parameters specified in `AudioTranscriptionRequest`. The audio source can be provided as a file path
    /// or as raw bytes with a file name.
    ///
    /// # Parameters
    ///
    /// - `request`: An `AudioTranscriptionRequest` containing details such as the audio source,
    ///   language, model, prompt, and other parameters.
    ///
    /// # Returns
    ///
    /// Returns an `AudioTranscriptionResponse` with the transcribed text if successful,
    /// or an `AudioTranscriptionError` in case of a failure.
    #[instrument(skip(self, request))]
    pub async fn audio_transcription(
        &self,
        request: AudioTranscriptionRequest,
    ) -> Result<AudioTranscriptionResponse, AudioTranscriptionError> {
        let mut form = multipart::Form::new()
            .text("model", request.model.to_string())
            .text("response_format", request.response_format.to_string());

        match request.source {
            FileSource::Filepath(file_path) => {
                let file_path = file_path.as_ref();

                if !file_path.exists() {
                    return Err(AudioTranscriptionError::FileNotFoundError(
                        file_path.to_string_lossy().into_owned(),
                    ));
                }

                form = form.file("file", file_path).await?;
            }
            FileSource::Bytes { buffer, file_name } => {
                let part = multipart::Part::bytes(buffer).file_name(file_name);
                form = form.part("file", part);
            }
        }

        if let Some(language) = request.language {
            form = form.text("language", language.to_string());
        }
        if let Some(prompt) = request.prompt {
            form = form.text("prompt", prompt.to_string());
        }
        if let Some(temperature) = request.temperature {
            form = form.text("temperature", temperature.to_string());
        }
        if let Some(timestamp_granularities) = request.timestamp_granularities {
            for granularity in timestamp_granularities {
                form = form.text("timestamp_granularities[]", granularity.to_string());
            }
        }

        let response = self
            .client
            .post(AUDIO_TRANSCRIPTION_API_URL)
            .multipart(form)
            .send()
            .await?
            .json::<AudioTranscriptionApiResponse>()
            .await?;

        match response {
            AudioTranscriptionApiResponse::TranscriptionResponse(response) => Ok(response),
            AudioTranscriptionApiResponse::ErrorResponse(error) => match error {
                ErrorResponse::Simple { detail } => {
                    Err(AudioTranscriptionError::ErrorResponse(detail))
                }
                ErrorResponse::Detailed { detail } => {
                    let error_details: Vec<String> = detail
                        .iter()
                        .map(|d| format!("{}: {}", d.loc.join("."), d.msg))
                        .collect();
                    Err(AudioTranscriptionError::ErrorResponse(
                        error_details.join(", "),
                    ))
                }
            },
        }
    }
}
