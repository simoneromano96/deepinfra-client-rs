pub mod client;

#[cfg(feature = "chat_completition")]
pub mod chat_completition;

#[cfg(feature = "audio_transcription")]
pub mod audio_transcription;

pub mod prelude;

pub use http;
pub use reqwest;
