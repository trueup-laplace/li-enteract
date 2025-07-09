# 🧠 Speech-to-Text with `whisper-rs` in Tauri

## Goal
When the user clicks the mic button in the control panel:
→ Record speech → Transcribe locally → Display in chat.

---

## Components

- **Microphone Input**: Capture user audio using MediaRecorder or Web Audio API.
- **File Handling**: Save recorded audio to a local path using Tauri’s filesystem API.
- **Transcription**: Use `whisper-rs` in the Rust backend to transcribe the saved audio.
- **Chat Display**: Update the frontend chat interface with the transcribed text.

---

## Notes

- No internet needed — runs fully offline.
- Model size impacts accuracy/speed — start with `small.en`.
- Ensure permissions are granted for mic access.

---
