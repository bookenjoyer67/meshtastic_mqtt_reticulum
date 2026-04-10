//! Audio message support with Codec2 and Opus encoding
//!
//! This module provides audio recording, encoding, and playback capabilities
//! with support for both Codec2 (for low-bandwidth LoRa/radio links) and
//! Opus (for higher quality audio).

#[cfg(feature = "audio")]
use codec2::Codec2;
#[cfg(feature = "audio")]
use opus::{Decoder, Encoder, Channels, Application};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::time::Duration;

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Enable audio support
    pub enabled: bool,
    /// Default audio codec (codec2 or opus)
    pub default_codec: AudioCodec,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of audio channels (1 for mono, 2 for stereo)
    pub channels: u16,
    /// Bitrate for Opus encoding (in kbps)
    pub opus_bitrate: u32,
    /// Codec2 mode (for low-bitrate encoding)
    pub codec2_mode: Codec2Mode,
    /// Maximum audio message duration in seconds
    pub max_duration: u32,
    /// Audio buffer size in samples
    pub buffer_size: usize,
}

/// Audio codec type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioCodec {
    Codec2(Codec2Mode),
    Opus,
    Raw,
}

/// Codec2 mode (bitrate and quality)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Codec2Mode {
    Mode700,  // 700 bps
    Mode1400, // 1400 bps  
    Mode2400, // 2400 bps
    Mode3200, // 3200 bps
}

impl Default for AudioConfig {
    fn default() -> Self {
        AudioConfig {
            enabled: true,
            default_codec: AudioCodec::Opus,
            sample_rate: 16000, // 16 kHz for voice
            channels: 1,        // Mono for voice
            opus_bitrate: 16000, // 16 kbps
            codec2_mode: Codec2Mode::Mode2400,
            max_duration: 60,   // 60 seconds max
            buffer_size: 1024,
        }
    }
}

/// Audio message
#[derive(Debug, Clone)]
pub struct AudioMessage {
    /// Audio data (encoded)
    pub data: Vec<u8>,
    /// Codec used
    pub codec: AudioCodec,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Original size before encoding (if known)
    pub original_size: Option<usize>,
}

/// Audio encoder/decoder
pub struct AudioProcessor {
    config: AudioConfig,
    #[cfg(feature = "audio")]
    opus_encoder: Option<Encoder>,
    #[cfg(feature = "audio")]
    opus_decoder: Option<Decoder>,
    #[cfg(feature = "audio")]
    codec2_encoder: Option<Codec2>,
    #[cfg(feature = "audio")]
    codec2_decoder: Option<Codec2>,
}

impl AudioProcessor {
    /// Create a new audio processor
    pub fn new(config: AudioConfig) -> Result<Self> {
        #[cfg(feature = "audio")]
        {
            let opus_encoder = if matches!(config.default_codec, AudioCodec::Opus) {
                Some(Encoder::new(
                    config.sample_rate,
                    Channels::Mono,
                    Application::Voip,
                )?)
            } else {
                None
            };

            let opus_decoder = if matches!(config.default_codec, AudioCodec::Opus) {
                Some(Decoder::new(config.sample_rate, Channels::Mono)?)
            } else {
                None
            };

            let codec2_encoder = if matches!(config.default_codec, AudioCodec::Codec2(_)) {
                // Codec2 doesn't have Mode enum, we need to use integer mode values
                // The mode is determined by the Codec2Mode enum
                let mode = match config.codec2_mode {
                    Codec2Mode::Mode700 => 0,  // 700 bps mode
                    Codec2Mode::Mode1400 => 1, // 1400 bps mode
                    Codec2Mode::Mode2400 => 2, // 2400 bps mode
                    Codec2Mode::Mode3200 => 3, // 3200 bps mode
                };
                Some(Codec2::new(mode))
            } else {
                None
            };

            let codec2_decoder = if matches!(config.default_codec, AudioCodec::Codec2(_)) {
                let mode = match config.codec2_mode {
                    Codec2Mode::Mode700 => 0,  // 700 bps mode
                    Codec2Mode::Mode1400 => 1, // 1400 bps mode
                    Codec2Mode::Mode2400 => 2, // 2400 bps mode
                    Codec2Mode::Mode3200 => 3, // 3200 bps mode
                };
                Some(Codec2::new(mode))
            } else {
                None
            };

            Ok(Self {
                config,
                opus_encoder,
                opus_decoder,
                codec2_encoder,
                codec2_decoder,
            })
        }

        #[cfg(not(feature = "audio"))]
        {
            Ok(Self { config })
        }
    }

    /// Encode audio data
    pub fn encode(&mut self, _pcm_data: &[i16], _codec: &AudioCodec) -> Result<AudioMessage> {
        #[cfg(feature = "audio")]
        {
            match codec {
                AudioCodec::Opus => {
                    if let Some(encoder) = &mut self.opus_encoder {
                        // Opus encode expects i16 samples
                        let mut encoded = vec![0; 4000]; // Initial buffer
                        let len = encoder.encode(pcm_data, &mut encoded)?;
                        encoded.truncate(len);
                        
                        let duration_ms = (pcm_data.len() as u32 * 1000) 
                            / (self.config.sample_rate * self.config.channels as u32);
                        
                        return Ok(AudioMessage {
                            data: encoded,
                            codec: codec.clone(),
                            duration_ms,
                            sample_rate: self.config.sample_rate,
                            channels: self.config.channels,
                            original_size: Some(pcm_data.len() * 2), // i16 = 2 bytes
                        });
                    }
                }
                AudioCodec::Codec2(_) => {
                    if let Some(encoder) = &mut self.codec2_encoder {
                        // Codec2 expects specific frame sizes
                        // This is simplified - real implementation would need proper framing
                        // Codec2::encode() doesn't return a value, it encodes in-place
                        // We need to handle this differently
                        let frame_size = encoder.samples_per_frame();
                        let mut encoded = Vec::new();
                        
                        // Process in frames
                        for chunk in pcm_data.chunks(frame_size) {
                            if chunk.len() == frame_size {
                                let mut bits = vec![0u8; encoder.bits_per_frame() / 8];
                                encoder.encode(&mut bits, chunk);
                                encoded.extend_from_slice(&bits);
                            }
                        }
                        
                        let duration_ms = (pcm_data.len() as u32 * 1000) 
                            / (self.config.sample_rate * self.config.channels as u32);
                        
                        return Ok(AudioMessage {
                            data: encoded,
                            codec: codec.clone(),
                            duration_ms,
                            sample_rate: self.config.sample_rate,
                            channels: self.config.channels,
                            original_size: Some(pcm_data.len() * 2),
                        });
                    }
                }
                AudioCodec::Raw => {
                    // Convert i16 to bytes
                    let mut data = Vec::with_capacity(pcm_data.len() * 2);
                    for &sample in pcm_data {
                        data.extend_from_slice(&sample.to_le_bytes());
                    }
                    
                    let duration_ms = (pcm_data.len() as u32 * 1000) 
                        / (self.config.sample_rate * self.config.channels as u32);
                    
                    return Ok(AudioMessage {
                        data,
                        codec: codec.clone(),
                        duration_ms,
                        sample_rate: self.config.sample_rate,
                        channels: self.config.channels,
                        original_size: Some(pcm_data.len() * 2),
                    });
                }
            }
        }

        // Fallback for no audio feature or unsupported codec
        Err(anyhow::anyhow!("Audio encoding not supported"))
    }

    /// Decode audio data
    pub fn decode(&mut self, _audio_msg: &AudioMessage) -> Result<Vec<i16>> {
        #[cfg(feature = "audio")]
        {
            match &audio_msg.codec {
                AudioCodec::Opus => {
                    if let Some(decoder) = &mut self.opus_decoder {
                        let frame_size = (audio_msg.duration_ms as usize * audio_msg.sample_rate as usize) / 1000;
                        let mut pcm_data = vec![0; frame_size];
                        let len = decoder.decode(&audio_msg.data, &mut pcm_data, false)?;
                        pcm_data.truncate(len);
                        return Ok(pcm_data);
                    }
                }
                AudioCodec::Codec2(_) => {
                    if let Some(decoder) = &mut self.codec2_decoder {
                        // Codec2::decode() doesn't return a value, it decodes in-place
                        // We need to handle this differently
                        let frame_size = decoder.samples_per_frame();
                        let bits_per_frame = decoder.bits_per_frame();
                        let bytes_per_frame = bits_per_frame / 8;
                        
                        let mut pcm_data = Vec::new();
                        
                        // Process in frames
                        for chunk in audio_msg.data.chunks(bytes_per_frame) {
                            if chunk.len() == bytes_per_frame {
                                let mut frame = vec![0i16; frame_size];
                                decoder.decode(&mut frame, chunk);
                                pcm_data.extend_from_slice(&frame);
                            }
                        }
                        
                        return Ok(pcm_data);
                    }
                }
                AudioCodec::Raw => {
                    // Convert bytes back to i16
                    let mut pcm_data = Vec::with_capacity(audio_msg.data.len() / 2);
                    for chunk in audio_msg.data.chunks_exact(2) {
                        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                        pcm_data.push(sample);
                    }
                    return Ok(pcm_data);
                }
            }
        }

        // Fallback
        Err(anyhow::anyhow!("Audio decoding not supported"))
    }

    /// Get estimated size for duration
    pub fn estimate_size(&self, duration_ms: u32, codec: &AudioCodec) -> usize {
        let samples = (duration_ms as usize * self.config.sample_rate as usize) / 1000;
        
        match codec {
            AudioCodec::Opus => {
                // Opus is variable bitrate, estimate based on configured bitrate
                let bits = (duration_ms as usize * self.config.opus_bitrate as usize) / 1000;
                bits / 8 // Convert to bytes
            }
            AudioCodec::Codec2(mode) => {
                let bps = match mode {
                    Codec2Mode::Mode700 => 700,
                    Codec2Mode::Mode1400 => 1400,
                    Codec2Mode::Mode2400 => 2400,
                    Codec2Mode::Mode3200 => 3200,
                };
                let bits = (duration_ms as usize * bps) / 1000;
                bits / 8
            }
            AudioCodec::Raw => {
                // Raw PCM: samples * channels * 2 bytes per sample (i16)
                samples * self.config.channels as usize * 2
            }
        }
    }

    /// Convert between codecs
    pub fn transcode(&mut self, audio_msg: &AudioMessage, target_codec: AudioCodec) -> Result<AudioMessage> {
        // Decode first
        let pcm_data = self.decode(audio_msg)?;
        
        // Then encode to target codec
        self.encode(&pcm_data, &target_codec)
    }
}

/// Audio recorder (simplified interface)
pub struct AudioRecorder {
    config: AudioConfig,
    processor: AudioProcessor,
}

impl AudioRecorder {
    /// Create a new audio recorder
    pub fn new(config: AudioConfig) -> Result<Self> {
        let processor = AudioProcessor::new(config.clone())?;
        Ok(Self { config, processor })
    }

    /// Record audio (simplified - would integrate with cpal in real implementation)
    pub fn record(&mut self, _duration: Duration, _codec: AudioCodec) -> Result<AudioMessage> {
        // In a real implementation, this would:
        // 1. Initialize audio input with cpal
        // 2. Record for specified duration
        // 3. Encode with specified codec
        
        // For now, return an error since this is a stub
        Err(anyhow::anyhow!("Audio recording not implemented"))
    }

    /// Play audio (simplified - would integrate with cpal in real implementation)
    pub fn play(&mut self, _audio_msg: &AudioMessage) -> Result<()> {
        // In a real implementation, this would:
        // 1. Decode audio
        // 2. Initialize audio output with cpal
        // 3. Play the decoded audio
        
        // For now, return an error since this is a stub
        Err(anyhow::anyhow!("Audio playback not implemented"))
    }
}

/// Audio message utilities
pub mod utils {
    use super::*;
    
    /// Create audio message from file
    pub fn audio_from_file(_path: &PathBuf, _codec: AudioCodec) -> Result<AudioMessage> {
        // In real implementation, would read WAV/MP3/etc. file
        // and encode with specified codec
        Err(anyhow::anyhow!("Audio file loading not implemented"))
    }
    
    /// Save audio message to file
    pub fn audio_to_file(_audio_msg: &AudioMessage, _path: &PathBuf) -> Result<()> {
        // In real implementation, would decode and save as WAV/MP3/etc.
        Err(anyhow::anyhow!("Audio file saving not implemented"))
    }
    
    /// Get audio duration from data
    pub fn get_audio_duration(audio_msg: &AudioMessage) -> Duration {
        Duration::from_millis(audio_msg.duration_ms as u64)
    }
    
    /// Calculate compression ratio
    pub fn compression_ratio(audio_msg: &AudioMessage) -> Option<f32> {
        if let Some(original_size) = audio_msg.original_size {
            if original_size > 0 {
                return Some(original_size as f32 / audio_msg.data.len() as f32);
            }
        }
        None
    }
}