# deepinfra-client-rs

A Rust client for interacting with Deepinfra APIs including chat completions and audio transcriptions.

## Overview

deepinfra-client-rs is a lightweight library that provides an easy way to communicate with Deepinfra's API endpoints for chat completions and audio transcriptions. It leverages popular crates like reqwest, serde, and tracing.

## Features

- **Chat Completions:** Supports generating completions using OpenAI's conversation style protocols.
- **Audio Transcriptions:** Enables conversion of audio files to text.
- **Modular Design:** Separated modules for chat completions, audio transcriptions, client building, and feature prelude.

## Installation

Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
deepinfra-client-rs = "0.0.1"
```

Or clone the repository from GitHub and build the project using Cargo.

## Usage

Import the required modules:

```rust
// ...existing code...
use deepinfra_client_rs::prelude::*;
// ...existing code...
```

Initialize the client:

```rust
// ...existing code...
let client = DeepinfraClient::new("your_token")?;
// ...existing code...
```

Check out the individual modules for detailed usage examples:
- `chat_completition`: For managing chat conversations.
- `audio_transcription`: For handling audio transcription requests.

## Advanced Usage

Below are quick examples to show how to perform chat completions and audio transcriptions.

### Chat Completions

```rust
use deepinfra_client_rs::prelude::*;
// ...existing code...

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = DeepinfraClient::new("your_token")?;
    let request = ChatCompletionRequestBuilder::default()
        .messages(vec![
            // Build your messages here...
        ])
        .build()
        .expect("Failed to build ChatCompletionRequest");

    let response = client.chat_completition(request).await?;
    println!("{:#?}", response);
    Ok(())
}
```

### Audio Transcriptions

```rust
use deepinfra_client_rs::prelude::*;
use deepinfra_client_rs::audio_transcription::{ FileSource, AudioTranscriptionRequestBuilder };
// ...existing code...

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = DeepinfraClient::new("your_token")?;
    let transcription_request = AudioTranscriptionRequestBuilder::default()
        .source(FileSource::Filepath("/path/to/audio.wav".into()))
        .build()
        .expect("Failed to build AudioTranscriptionRequest");

    let response = client.audio_transcription(transcription_request).await?;
    println!("Transcribed text: {}", response.text);
    Ok(())
}
```

