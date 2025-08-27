//! Audio processing utilities for format conversion, resampling, and analysis

use crate::types::AudioCaptureResult;

/// Audio processing utilities
pub struct AudioProcessor;

impl AudioProcessor {
    /// Resample audio from one sample rate to another using linear interpolation
    pub fn resample_audio(
        input_samples: &[f32],
        input_rate: u32,
        output_rate: u32,
    ) -> AudioCaptureResult<Vec<f32>> {
        if input_rate == output_rate {
            return Ok(input_samples.to_vec());
        }
        
        let ratio = output_rate as f32 / input_rate as f32;
        let output_count = (input_samples.len() as f32 * ratio) as usize;
        let mut output_samples = Vec::with_capacity(output_count);
        
        for i in 0..output_count {
            let input_index = i as f32 / ratio;
            let input_index_int = input_index as usize;
            
            if input_index_int >= input_samples.len() - 1 {
                // Handle edge case
                output_samples.push(*input_samples.last().unwrap_or(&0.0));
            } else {
                // Linear interpolation
                let fraction = input_index - input_index_int as f32;
                let sample1 = input_samples[input_index_int];
                let sample2 = input_samples[input_index_int + 1];
                let interpolated = sample1 + fraction * (sample2 - sample1);
                output_samples.push(interpolated);
            }
        }
        
        Ok(output_samples)
    }
    
    /// Convert stereo audio to mono by averaging channels
    pub fn stereo_to_mono(stereo_samples: &[f32]) -> AudioCaptureResult<Vec<f32>> {
        if stereo_samples.len() % 2 != 0 {
            return Err(crate::types::AudioCaptureError::InvalidConfiguration(
                "Stereo samples must have even length".to_string()
            ));
        }
        
        let mono_samples: Vec<f32> = stereo_samples
            .chunks_exact(2)
            .map(|chunk| (chunk[0] + chunk[1]) * 0.5)
            .collect();
        
        Ok(mono_samples)
    }
    
    /// Convert mono audio to stereo by duplicating the channel
    pub fn mono_to_stereo(mono_samples: &[f32]) -> AudioCaptureResult<Vec<f32>> {
        let mut stereo_samples = Vec::with_capacity(mono_samples.len() * 2);
        
        for &sample in mono_samples {
            stereo_samples.push(sample); // Left channel
            stereo_samples.push(sample); // Right channel
        }
        
        Ok(stereo_samples)
    }
    
    /// Convert float32 samples to int16 samples
    pub fn float32_to_int16(float_samples: &[f32]) -> AudioCaptureResult<Vec<i16>> {
        let int16_samples: Vec<i16> = float_samples
            .iter()
            .map(|&sample| {
                let clamped = sample.clamp(-1.0, 1.0);
                (clamped * 32767.0) as i16
            })
            .collect();
        
        Ok(int16_samples)
    }
    
    /// Convert int16 samples to float32 samples
    pub fn int16_to_float32(int16_samples: &[i16]) -> AudioCaptureResult<Vec<f32>> {
        let float_samples: Vec<f32> = int16_samples
            .iter()
            .map(|&sample| sample as f32 / 32767.0)
            .collect();
        
        Ok(float_samples)
    }
    
    /// Calculate RMS (Root Mean Square) level of audio samples
    pub fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        
        let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }
    
    /// Calculate peak level of audio samples
    pub fn calculate_peak(samples: &[f32]) -> f32 {
        samples.iter().map(|&x| x.abs()).fold(0.0, f32::max)
    }
    
    /// Calculate audio level in dB
    pub fn calculate_level_db(samples: &[f32]) -> f32 {
        let rms = Self::calculate_rms(samples);
        if rms <= 0.0 {
            -96.0 // Very quiet
        } else {
            20.0 * rms.log10()
        }
    }
    
    /// Apply a simple low-pass filter to reduce noise
    pub fn apply_low_pass_filter(samples: &[f32], cutoff_freq: f32, sample_rate: u32) -> AudioCaptureResult<Vec<f32>> {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
        let dt = 1.0 / sample_rate as f32;
        let alpha = dt / (rc + dt);
        
        let mut filtered_samples = Vec::with_capacity(samples.len());
        let mut last_sample = 0.0;
        
        for &sample in samples {
            let filtered = alpha * sample + (1.0 - alpha) * last_sample;
            filtered_samples.push(filtered);
            last_sample = filtered;
        }
        
        Ok(filtered_samples)
    }
    
    /// Normalize audio samples to a target peak level
    pub fn normalize_audio(samples: &[f32], target_peak: f32) -> AudioCaptureResult<Vec<f32>> {
        let current_peak = Self::calculate_peak(samples);
        if current_peak <= 0.0 {
            return Ok(samples.to_vec());
        }
        
        let scale_factor = target_peak / current_peak;
        let normalized_samples: Vec<f32> = samples
            .iter()
            .map(|&sample| sample * scale_factor)
            .collect();
        
        Ok(normalized_samples)
    }
    
    /// Detect silence in audio samples
    pub fn detect_silence(samples: &[f32], threshold: f32) -> bool {
        let rms = Self::calculate_rms(samples);
        rms < threshold
    }
    
    /// Split audio into chunks of specified duration
    pub fn split_into_chunks(
        samples: &[f32],
        sample_rate: u32,
        chunk_duration_seconds: f32,
    ) -> AudioCaptureResult<Vec<Vec<f32>>> {
        let chunk_size = (sample_rate as f32 * chunk_duration_seconds) as usize;
        let mut chunks = Vec::new();
        
        for chunk in samples.chunks(chunk_size) {
            chunks.push(chunk.to_vec());
        }
        
        Ok(chunks)
    }
    
    /// Pad or truncate audio to a specific length
    pub fn pad_or_truncate(samples: &[f32], target_length: usize) -> AudioCaptureResult<Vec<f32>> {
        let mut result = samples.to_vec();
        
        if result.len() < target_length {
            // Pad with zeros
            result.extend(std::iter::repeat(0.0).take(target_length - result.len()));
        } else if result.len() > target_length {
            // Truncate
            result.truncate(target_length);
        }
        
        Ok(result)
    }
}

/// Audio format conversion utilities
pub mod format_converter {
    use super::*;
    
    /// Convert audio data between different formats
    pub struct FormatConverter;
    
    impl FormatConverter {
        /// Convert raw bytes to float32 samples
        /// 
        /// # TODO: Unsigned Integer Support
        /// 
        /// This function currently assumes all integer audio data is signed (i16, i32).
        /// Core Audio supports both signed and unsigned integer formats, and some audio
        /// hardware (especially legacy systems) may output unsigned PCM data (u8, u16).
        /// 
        /// When implementing unsigned support, consider:
        /// - 8-bit unsigned: range 0-255, center at 128
        /// - 16-bit unsigned: range 0-65535, center at 32768
        /// - Conversion: `signed = unsigned - center_value`
        /// - Core Audio format flags: `AUDIO_FORMAT_FLAG_IS_SIGNED_INTEGER`
        /// 
        /// This is important for compatibility with:
        /// - Legacy audio hardware
        /// - Embedded audio systems
        /// - Certain audio file formats
        /// - Future audio sources that may use unsigned formats
        pub fn bytes_to_float32(
            bytes: &[u8],
            bits_per_sample: u16,
            channels: u16,
            is_little_endian: bool,
        ) -> AudioCaptureResult<Vec<f32>> {
            match bits_per_sample {
                16 => {
                    let samples = if is_little_endian {
                        bytes.chunks_exact(2)
                            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                            .collect::<Vec<i16>>()
                    } else {
                        bytes.chunks_exact(2)
                            .map(|chunk| i16::from_be_bytes([chunk[0], chunk[1]]))
                            .collect::<Vec<i16>>()
                    };
                    
                    let float_samples = AudioProcessor::int16_to_float32(&samples)?;
                    
                    if channels == 2 {
                        AudioProcessor::stereo_to_mono(&float_samples)
                    } else {
                        Ok(float_samples)
                    }
                }
                32 => {
                    let samples = if is_little_endian {
                        bytes.chunks_exact(4)
                            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                            .collect::<Vec<f32>>()
                    } else {
                        bytes.chunks_exact(4)
                            .map(|chunk| f32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                            .collect::<Vec<f32>>()
                    };
                    
                    if channels == 2 {
                        AudioProcessor::stereo_to_mono(&samples)
                    } else {
                        Ok(samples)
                    }
                }
                _ => Err(crate::types::AudioCaptureError::InvalidConfiguration(
                    format!("Unsupported bits per sample: {}", bits_per_sample)
                )),
            }
        }
        
        /// Convert float32 samples to raw bytes
        /// 
        /// # TODO: Unsigned Integer Support
        /// 
        /// This function currently always outputs signed integer formats (i16, i32).
        /// When implementing unsigned support, consider:
        /// - 8-bit unsigned: range 0-255, center at 128
        /// - 16-bit unsigned: range 0-65535, center at 32768
        /// - Conversion: `unsigned = signed + center_value`
        /// - Core Audio format flags: `AUDIO_FORMAT_FLAG_IS_SIGNED_INTEGER`
        pub fn float32_to_bytes(
            samples: &[f32],
            bits_per_sample: u16,
            channels: u16,
            is_little_endian: bool,
        ) -> AudioCaptureResult<Vec<u8>> {
            let mut bytes = Vec::new();
            
            match bits_per_sample {
                16 => {
                    let int16_samples = AudioProcessor::float32_to_int16(samples)?;
                    
                    if channels == 2 {
                        let stereo_samples = AudioProcessor::mono_to_stereo(samples)?;
                        let stereo_int16 = AudioProcessor::float32_to_int16(&stereo_samples)?;
                        
                        for &sample in &stereo_int16 {
                            let sample_bytes = if is_little_endian {
                                sample.to_le_bytes()
                            } else {
                                sample.to_be_bytes()
                            };
                            bytes.extend_from_slice(&sample_bytes);
                        }
                    } else {
                        for &sample in &int16_samples {
                            let sample_bytes = if is_little_endian {
                                sample.to_le_bytes()
                            } else {
                                sample.to_be_bytes()
                            };
                            bytes.extend_from_slice(&sample_bytes);
                        }
                    }
                }
                32 => {
                    if channels == 2 {
                        let stereo_samples = AudioProcessor::mono_to_stereo(samples)?;
                        
                        for &sample in &stereo_samples {
                            let sample_bytes = if is_little_endian {
                                sample.to_le_bytes()
                            } else {
                                sample.to_be_bytes()
                            };
                            bytes.extend_from_slice(&sample_bytes);
                        }
                    } else {
                        for &sample in samples {
                            let sample_bytes = if is_little_endian {
                                sample.to_le_bytes()
                            } else {
                                sample.to_be_bytes()
                            };
                            bytes.extend_from_slice(&sample_bytes);
                        }
                    }
                }
                _ => {
                    return Err(crate::types::AudioCaptureError::InvalidConfiguration(
                        format!("Unsupported bits per sample: {}", bits_per_sample)
                    ));
                }
            }
            
            Ok(bytes)
        }
    }
}
