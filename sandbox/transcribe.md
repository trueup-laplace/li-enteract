# 🚀 WASAPI + Whisper Real-time Transcription Setup

## 📥 Download Whisper Model

You'll need to download a Whisper model file to enable transcription:

### Option 1: Tiny Model (Fastest - Recommended for Real-time)
```bash
# Create models directory
mkdir models

# Download tiny model (39 MB) - fastest for real-time
curl -L -o models/ggml-tiny.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin
```

### Option 2: Base Model (Better Accuracy)
```bash
# Download base model (142 MB) - better accuracy but slower
curl -L -o models/ggml-base.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

### Option 3: Small Model (Balanced)
```bash
# Download small model (244 MB) - good balance
curl -L -o models/ggml-small.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
```

## 🔧 Build the Project

```bash
# Build with Whisper support
cargo build --release

# Or build in debug mode for development
cargo build
```

## 🎤 Run Real-time Transcription

### With Tiny Model (Recommended)
```bash
cargo run --bin audio_capture models/ggml-tiny.bin
```

### With Different Models
```bash
# Using base model
cargo run --bin audio_capture models/ggml-base.bin

# Using small model  
cargo run --bin audio_capture models/ggml-small.bin

# Model in current directory
cargo run --bin audio_capture ggml-tiny.bin
```

### Audio Capture Only (No Transcription)
```bash
# Just audio capture without Whisper
cargo run --bin audio_capture
```

## 📊 Expected Performance

| Model | Size | Speed (RTF) | Accuracy | Use Case |
|-------|------|-------------|----------|----------|
| Tiny  | 39MB | 0.3-0.6x    | Good     | Real-time transcription |
| Base  | 142MB| 0.6-1.2x    | Better   | Near real-time |
| Small | 244MB| 1.0-2.0x    | Best     | Offline processing |

**RTF (Real-time Factor)**: Lower is better. <1.0x means faster than real-time.

## 🎯 What You'll See

### Device Selection
```
🎯 Selected device: Speakers (Realtek(R) Audio) [Render via RenderLoopback]
🤖 Using Whisper model: models/ggml-tiny.bin
✅ Whisper transcriber ready
```

### Real-time Transcription Output
```
⚡ [12:34:56.789] Hello, this is a test transcription (0.85) (0.4s) (0.6x)
🚀 [12:34:58.123] The audio quality is very good today (0.92) (0.3s) (0.4x)
🎯 [12:35:00.456] Real-time factor is excellent (0.78) (0.5s) (0.7x)
```

### Performance Statistics
```
=== TRANSCRIPTION STATISTICS ===
Total Transcriptions: 45
Average Processing Time: 0.42s
Average Real-time Factor: 0.61x
🚀 ACHIEVED REAL-TIME PERFORMANCE!
```

## 🔧 Troubleshooting

### "No Whisper model found"
- Download a model file using the commands above
- Verify the file path is correct
- Check file size matches expected size

### "Whisper initialization failed"
- Ensure you have enough RAM (models load into memory)
- Try a smaller model (tiny instead of base/small)
- Check file isn't corrupted (re-download if needed)

### Slow transcription (RTF > 1.0x)
- Use the tiny model for real-time performance
- Close other applications to free up CPU
- Try running with `--release` build for better performance

### No audio being captured
- Follow the audio troubleshooting from the main README
- Ensure you're playing audio while testing
- Check Windows audio levels

## 🎯 Performance Optimization Tips

1. **Use Release Build**: `cargo build --release` for ~3x speed improvement
2. **Tiny Model**: Use ggml-tiny.bin for best real-time performance  
3. **Close Applications**: Free up CPU for Whisper processing
4. **Good Audio**: Clear audio improves transcription accuracy and speed

## 📝 Output Files

The program creates these files:
- `captured_audio.raw` - Raw 16kHz mono audio data
- `realtime_transcription_log.txt` - Complete transcription log with timestamps

## 🎉 Success Criteria

You'll know it's working when you see:
- ✅ Real-time factor (RTF) < 1.0x consistently
- 🚀 Speed indicators (⚡🚀🎯) showing fast processing  
- 📝 Accurate transcriptions appearing within 1-2 seconds of speech
- 📊 Audio levels changing as you play different content

Enjoy your real-time Rust + Whisper transcription system! 🎤✨