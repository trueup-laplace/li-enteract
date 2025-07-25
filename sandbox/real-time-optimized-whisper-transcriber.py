#!/usr/bin/env python3
"""
REAL-TIME OPTIMIZED Whisper Transcription
Using tiny model with speed optimizations while maintaining audio quality
"""

import sys
import time
import threading
import queue
import numpy as np
from collections import deque
import os
from datetime import datetime

try:
    import pyaudiowpatch as pyaudio
except ImportError:
    print("Error: pyaudiowpatch not found. Install with: pip install pyaudiowpatch")
    sys.exit(1)

try:
    from faster_whisper import WhisperModel
except ImportError:
    print("Error: faster-whisper not found. Install with: pip install faster-whisper")
    sys.exit(1)

try:
    import scipy.signal
    import scipy.ndimage
except ImportError:
    print("Error: scipy not found. Install with: pip install scipy")
    sys.exit(1)

# REAL-TIME OPTIMIZED Configuration
WHISPER_SAMPLE_RATE = 16000
CHUNK_SIZE = 4096
CHANNELS = 2
FORMAT = pyaudio.paInt16
BUFFER_DURATION = 4.0       # REDUCED for faster processing
OVERLAP_DURATION = 1.0      # REDUCED overlap
MIN_AUDIO_LENGTH = 1.5      # SHORTER minimum length
MIN_CONFIDENCE = 0.35       # SLIGHTLY lower for speed
PROCESSING_INTERVAL = 0.8   # FASTER processing trigger

class RealtimeWhisperTranscriber:
    def __init__(self, model_size="tiny", log_file="realtime_transcription_log.txt"):
        self.whisper_sample_rate = WHISPER_SAMPLE_RATE
        self.chunk_size = CHUNK_SIZE
        self.is_running = False
        self.device_sample_rate = None
        self.log_file = log_file
        self.model_size = model_size
        
        # OPTIMIZED buffer sizes
        self.buffer_size = int(BUFFER_DURATION * self.whisper_sample_rate)
        self.overlap_size = int(OVERLAP_DURATION * self.whisper_sample_rate)
        self.min_audio_samples = int(MIN_AUDIO_LENGTH * self.whisper_sample_rate)
        
        print(f"REAL-TIME Buffer configuration:")
        print(f"  Buffer duration: {BUFFER_DURATION}s ({self.buffer_size} samples)")
        print(f"  Overlap duration: {OVERLAP_DURATION}s ({self.overlap_size} samples)")
        print(f"  Minimum audio length: {MIN_AUDIO_LENGTH}s ({self.min_audio_samples} samples)")
        print(f"  Processing interval: {PROCESSING_INTERVAL}s")
        
        # Initialize log file
        self.setup_logging()
        
        # Initialize Whisper with SPEED settings
        print(f"Loading Whisper model '{model_size}' with SPEED optimizations...")
        try:
            self.model = WhisperModel(
                model_size, 
                device="cpu", 
                compute_type="int8",      # FASTER: int8 instead of float32
                cpu_threads=6,            # MORE threads for tiny model
                num_workers=1
            )
            print("✓ Speed-optimized Whisper model loaded")
        except Exception as e:
            print(f"❌ Error loading Whisper model: {e}")
            sys.exit(1)
        
        # Audio processing
        self.audio_queue = queue.Queue(maxsize=100)  # Limit queue size
        self.audio_buffer = deque(maxlen=self.buffer_size * 2)
        self.last_transcription_time = 0
        self.processing_lock = threading.Lock()
        
        # Speed tracking
        self.dc_offset_history = deque(maxlen=5)  # Smaller history
        self.level_history = deque(maxlen=5)
        
        # Performance statistics
        self.total_chunks_processed = 0
        self.total_transcriptions = 0
        self.total_processing_time = 0
        self.processing_times = deque(maxlen=10)
        self.start_time = time.time()
        
    def setup_logging(self):
        with open(self.log_file, 'w', encoding='utf-8') as f:
            f.write(f"=== REAL-TIME Whisper Transcription Log Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')} ===\n\n")
        print(f"✓ Logging to: {self.log_file}")
    
    def log_transcription(self, text, confidence=None, processing_time=None, real_time_factor=None):
        timestamp = datetime.now().strftime('%H:%M:%S.%f')[:-3]
        
        log_parts = [f"[{timestamp}] TRANSCRIPTION: {text}"]
        if confidence is not None:
            log_parts.append(f"(conf: {confidence:.3f})")
        if processing_time is not None:
            log_parts.append(f"({processing_time:.2f}s)")
        if real_time_factor is not None:
            log_parts.append(f"(RTF: {real_time_factor:.2f}x)")
        
        log_entry = " ".join(log_parts) + "\n"
        
        try:
            with open(self.log_file, 'a', encoding='utf-8') as f:
                f.write(log_entry)
        except Exception as e:
            print(f"Error writing to log: {e}")
        
        # Console output with speed indicator
        speed_indicator = "⚡" if real_time_factor and real_time_factor < 0.5 else "🚀" if real_time_factor and real_time_factor < 1.0 else "🎯"
        console_parts = [f"{speed_indicator} [{timestamp}] {text}"]
        if confidence is not None:
            console_parts.append(f"({confidence:.3f})")
        if processing_time is not None:
            console_parts.append(f"({processing_time:.2f}s)")
        if real_time_factor is not None:
            console_parts.append(f"({real_time_factor:.2f}x)")
        
        print(" ".join(console_parts))
    
    def find_best_loopback_device(self):
        """Streamlined device detection."""
        p = pyaudio.PyAudio()
        devices = []
        
        print("\n=== FAST DEVICE SCAN ===")
        for i in range(p.get_device_count()):
            device_info = p.get_device_info_by_index(i)
            if "loopback" in device_info['name'].lower() and device_info['maxInputChannels'] > 0:
                devices.append((i, device_info))
                print(f"  {len(devices)}. {device_info['name']}")
        
        p.terminate()
        
        if not devices:
            print("No loopback devices found!")
            return None
        
        # Auto-select first working device for speed
        for device_idx, device_info in devices:
            print(f"\nTesting: {device_info['name']}")
            
            for rate in [48000, 44100, 16000]:
                if self.test_device_quality(device_idx, rate):
                    print(f"✓ Auto-selected: {device_info['name']} at {rate} Hz")
                    self.device_sample_rate = rate
                    return device_idx
                    
        return None
    
    def test_device_quality(self, device_index, sample_rate):
        """Quick device test."""
        p = pyaudio.PyAudio()
        try:
            test_stream = p.open(
                format=FORMAT, channels=CHANNELS, rate=sample_rate,
                input=True, input_device_index=device_index, frames_per_buffer=self.chunk_size
            )
            data = test_stream.read(self.chunk_size, exception_on_overflow=False)
            test_stream.close()
            p.terminate()
            return True
        except:
            p.terminate()
            return False
    
    def fast_audio_process(self, raw_audio_data):
        """OPTIMIZED audio processing for speed."""
        audio_array = np.frombuffer(raw_audio_data, dtype=np.int16)
        
        # FAST stereo handling
        if CHANNELS == 2:
            stereo_audio = audio_array.reshape(-1, 2)
            left_channel = stereo_audio[:, 0]
            right_channel = stereo_audio[:, 1]
            
            # Quick stereo check
            stereo_diff = np.mean(np.abs(left_channel.astype(np.float32) - right_channel.astype(np.float32)))
            
            if stereo_diff > 200:
                # True stereo - use simple RMS comparison
                left_rms = np.mean(left_channel.astype(np.float32)**2)
                right_rms = np.mean(right_channel.astype(np.float32)**2)
                audio_mono = left_channel if left_rms > right_rms else right_channel
            else:
                # Mono in stereo format
                audio_mono = left_channel
        else:
            audio_mono = audio_array
        
        # FAST DC offset removal (only if significant)
        dc_offset = np.mean(audio_mono.astype(np.float32))
        if abs(dc_offset) > 100:  # Only remove if significant
            audio_mono = audio_mono - int(dc_offset)
        
        # FAST resampling (simplified)
        if self.device_sample_rate != self.whisper_sample_rate:
            # Use simple decimation for common rates
            if self.device_sample_rate == 48000 and self.whisper_sample_rate == 16000:
                # Simple 3:1 decimation
                audio_mono = audio_mono[::3]
            elif self.device_sample_rate == 44100 and self.whisper_sample_rate == 16000:
                # Approximate decimation
                factor = self.device_sample_rate / self.whisper_sample_rate
                indices = np.arange(0, len(audio_mono), factor).astype(int)
                audio_mono = audio_mono[indices]
            else:
                # Fallback to scipy (slower but accurate)
                audio_float = audio_mono.astype(np.float32)
                num_samples_out = int(len(audio_float) * self.whisper_sample_rate / self.device_sample_rate)
                resampled = scipy.signal.resample(audio_float, num_samples_out)
                audio_mono = np.clip(resampled, -32768, 32767).astype(np.int16)
        
        return audio_mono
    
    def audio_callback(self, in_data, frame_count, time_info, status):
        if status:
            print(f"Audio status: {status}")
        
        # Non-blocking queue add
        try:
            self.audio_queue.put_nowait(in_data)
        except queue.Full:
            # Drop oldest if queue is full (real-time priority)
            try:
                self.audio_queue.get_nowait()
                self.audio_queue.put_nowait(in_data)
            except queue.Empty:
                pass
        
        return (None, pyaudio.paContinue)
    
    def process_audio_stream(self):
        """REAL-TIME audio processing loop."""
        print("⚡ Starting REAL-TIME audio processing...")
        
        last_level_time = time.time()
        processing_thread = None
        
        while self.is_running:
            try:
                raw_audio_data = self.audio_queue.get(timeout=0.1)
                
                # FAST audio processing
                processed_audio = self.fast_audio_process(raw_audio_data)
                
                # Add to buffer
                self.audio_buffer.extend(processed_audio)
                self.total_chunks_processed += 1
                
                # FAST monitoring (less frequent)
                current_time = time.time()
                if current_time - last_level_time > 5.0:  # Every 5 seconds
                    level = self.calculate_audio_level(processed_audio)
                    avg_processing_time = np.mean(self.processing_times) if self.processing_times else 0
                    
                    print(f"⚡ Audio: {level:.1f} dB | Buffer: {len(self.audio_buffer)} | Avg Process: {avg_processing_time:.2f}s")
                    last_level_time = current_time
                
                # AGGRESSIVE processing trigger
                if (len(self.audio_buffer) >= self.buffer_size and 
                    (processing_thread is None or not processing_thread.is_alive()) and
                    current_time - self.last_transcription_time > PROCESSING_INTERVAL):
                    
                    processing_thread = threading.Thread(target=self.process_buffer_fast, daemon=True)
                    processing_thread.start()
                    self.last_transcription_time = current_time
                    
            except queue.Empty:
                continue
            except Exception as e:
                print(f"Error in audio processing: {e}")
    
    def calculate_audio_level(self, audio_data):
        """Fast audio level calculation."""
        if len(audio_data) == 0:
            return -60
        rms = np.sqrt(np.mean(audio_data.astype(np.float32) ** 2))
        return max(20 * np.log10(rms / 32768.0), -60) if rms > 0 else -60
    
    def process_buffer_fast(self):
        """SPEED-OPTIMIZED Whisper processing."""
        with self.processing_lock:
            try:
                # Get audio data
                audio_data = np.array(list(self.audio_buffer), dtype=np.int16)
                
                if len(audio_data) < self.min_audio_samples:
                    return
                
                # Use smaller chunk for speed
                max_samples = min(len(audio_data), self.buffer_size)
                audio_data = audio_data[-max_samples:]
                
                # Quick quality check
                rms = np.sqrt(np.mean(audio_data.astype(np.float32)**2))
                if rms < 100:
                    return
                
                # Convert to float32
                audio_float = audio_data.astype(np.float32) / 32768.0
                audio_duration = len(audio_data) / self.whisper_sample_rate
                
                # SPEED-OPTIMIZED Whisper settings
                start_time = time.time()
                
                segments, info = self.model.transcribe(
                    audio_float,
                    language=None,
                    beam_size=1,               # FASTEST: single beam
                    best_of=1,                 # FASTEST: single candidate
                    temperature=0.0,
                    condition_on_previous_text=False,  # FASTER: no context
                    vad_filter=False,
                    word_timestamps=False,
                    compression_ratio_threshold=2.4,
                    log_prob_threshold=-1.0,
                    suppress_blank=True,
                    without_timestamps=True
                )
                
                processing_time = time.time() - start_time
                real_time_factor = processing_time / audio_duration
                
                self.processing_times.append(processing_time)
                self.total_processing_time += processing_time
                
                # Extract transcription
                transcription_parts = []
                confidences = []
                
                for segment in segments:
                    text = segment.text.strip()
                    if text and len(text) > 1:
                        transcription_parts.append(text)
                        if hasattr(segment, 'avg_logprob'):
                            confidences.append(segment.avg_logprob)
                
                if transcription_parts:
                    full_text = " ".join(transcription_parts)
                    
                    # Calculate confidence
                    confidence = None
                    if confidences:
                        avg_log_prob = np.mean(confidences)
                        confidence = np.exp(avg_log_prob)
                    
                    # RELAXED quality filtering for speed
                    if self.is_fast_quality_ok(full_text, confidence):
                        self.log_transcription(
                            full_text, 
                            confidence=confidence, 
                            processing_time=processing_time,
                            real_time_factor=real_time_factor
                        )
                        self.total_transcriptions += 1
                    else:
                        print(f"⚡ Filtered low quality (conf: {confidence:.3f if confidence else 'N/A'})")
                
                # AGGRESSIVE buffer management for real-time
                samples_to_remove = len(audio_data) - self.overlap_size
                for _ in range(min(samples_to_remove, len(self.audio_buffer))):
                    self.audio_buffer.popleft()
                
            except Exception as e:
                print(f"Error in fast Whisper processing: {e}")
    
    def is_fast_quality_ok(self, text, confidence):
        """FAST quality check."""
        if not text or len(text) < 2:
            return False
        
        if confidence is not None and confidence < MIN_CONFIDENCE:
            return False
        
        # Simple repetition check
        words = text.split()
        if len(words) > 4:
            unique_ratio = len(set(words)) / len(words)
            if unique_ratio < 0.3:
                return False
        
        return True
    
    def start_transcription(self):
        """Start REAL-TIME transcription."""
        device_index = self.find_best_loopback_device()
        if device_index is None:
            print("❌ No suitable audio device found.")
            return
        
        print(f"\n=== Starting REAL-TIME Transcription ===")
        print(f"Model: {self.model_size} (speed optimized)")
        print(f"Compute: int8 (faster)")
        print(f"Processing: Every {PROCESSING_INTERVAL}s")
        print(f"Target: <1.0x real-time factor")
        print("=" * 50)
        
        p = pyaudio.PyAudio()
        
        try:
            stream = p.open(
                format=FORMAT,
                channels=CHANNELS,
                rate=self.device_sample_rate,
                input=True,
                input_device_index=device_index,
                frames_per_buffer=self.chunk_size,
                stream_callback=self.audio_callback
            )
            
            print("✓ Real-time audio stream opened")
            
            self.is_running = True
            audio_thread = threading.Thread(target=self.process_audio_stream, daemon=True)
            audio_thread.start()
            
            stream.start_stream()
            print("✓ REAL-TIME processing started")
            print("\n⚡ LISTENING in REAL-TIME MODE...")
            print("🚀 Optimized for speed with tiny model + int8")
            print("📊 Watch for real-time factor (lower = faster)")
            print("Press Ctrl+C to stop\n")
            
            try:
                while stream.is_active():
                    time.sleep(0.1)
            except KeyboardInterrupt:
                print("\n⏹️ Stopping...")
                
        except Exception as e:
            print(f"❌ Error: {e}")
            import traceback
            traceback.print_exc()
            
        finally:
            self.is_running = False
            if 'stream' in locals():
                stream.stop_stream()
                stream.close()
            p.terminate()
            
            # Performance statistics
            runtime = time.time() - self.start_time
            avg_processing_time = self.total_processing_time / max(1, self.total_transcriptions)
            avg_rtf = np.mean(self.processing_times) / (BUFFER_DURATION * 0.8) if self.processing_times else 0
            
            print(f"\n=== REAL-TIME PERFORMANCE STATS ===")
            print(f"Runtime: {runtime:.1f}s")
            print(f"Transcriptions: {self.total_transcriptions}")
            print(f"Average processing time: {avg_processing_time:.2f}s")
            print(f"Average real-time factor: {avg_rtf:.2f}x")
            print(f"Processing rate: {self.total_chunks_processed/runtime:.1f} chunks/sec")
            
            if avg_rtf < 1.0:
                print("🚀 ACHIEVED REAL-TIME PERFORMANCE!")
            elif avg_rtf < 1.5:
                print("⚡ NEAR REAL-TIME PERFORMANCE")
            else:
                print("🔧 Consider further optimizations")
            
            print("✓ Session complete")

def main():
    print("=== REAL-TIME Whisper Streaming Transcription ===")
    print("Speed-optimized with tiny model for real-time performance")
    print("Target: Sub-real-time processing with good accuracy\n")
    
    transcriber = RealtimeWhisperTranscriber("tiny")
    
    try:
        transcriber.start_transcription()
    except KeyboardInterrupt:
        print("\nExiting...")
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()