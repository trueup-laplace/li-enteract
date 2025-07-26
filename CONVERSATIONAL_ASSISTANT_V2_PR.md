# Conversational Assistant V2 - Live Response System

## Overview
This PR introduces a comprehensive conversational assistant system with real-time audio transcription, live AI response suggestions, and enhanced UI/UX improvements throughout the application.

## Key Features

### 🎤 Conversational Window
- **New dedicated conversation interface** for real-time transcription and interaction
- **Dual audio input system**: Microphone + System audio loopback capture
- **Live transcription display** with typing indicators and preview messages
- **Session management** with conversation history persistence
- **Export functionality** for conversation transcripts

### 🤖 Live AI Response Assistant
- **Real-time response suggestions** that analyze conversation context
- **Smart debouncing** (3s pause detection, 5s minimum between analyses)
- **Suggestion history** maintains up to 5 recent suggestions
- **Context-aware assistance** triggered when system detects others speaking
- **Copy-to-clipboard** functionality for easy use of suggestions

### 🔊 Audio Loopback System
- **WASAPI-based audio capture** for Windows system audio
- **Automatic device detection** and best device selection
- **Multiple capture methods**: Render loopback, Stereo Mix, Capture devices
- **Quality filtering** with noise suppression and audio validation
- **Configurable buffer settings** for optimal performance

### 🎨 UI/UX Enhancements

#### Live AI Interface
- **Floating stop button** positioned at bottom-right
- **Auto-scrolling suggestions** that stick to newest entries
- **Visual feedback** with processing indicators and status displays
- **Responsive design** with glass morphism effects

#### Settings Panel Redesign
- **Modern navigation sidebar** with gradient backgrounds and animations
- **Enhanced status indicators** with animated pulse effects
- **Premium card design** for models and audio devices
- **Color-coded model badges** (Recommended, Vision, Research)
- **Improved visual hierarchy** and micro-interactions

### 🛠️ Technical Improvements

#### Backend (Rust)
- **New audio loopback module** with device enumeration and capture
- **Ollama integration** for conversational AI model (`gemma3:1b-it-qat`)
- **Streaming responses** with real-time event emission
- **Conversation persistence** with automatic session saving

#### Frontend (Vue 3)
- **New composables**: `useLiveAI`, `useAudioLoopback`, `useConversationManagement`
- **Pinia stores**: `conversation.ts` for state management
- **Component architecture**: Modular design with reusable components
- **Event-driven updates** with proper debouncing and throttling

## File Changes

### New Components
- `ConversationalWindow.vue` - Main conversation interface
- `LiveAI.vue` - Live response assistant drawer
- `AIAssistant.vue` - AI query interface
- `MessageList.vue` & `MessageItem.vue` - Conversation display
- `ConversationSidebar.vue` - Session management
- `ExportControls.vue` - Export functionality
- `AudioLoopbackControl.vue` - Audio device controls

### New Backend Modules
- `audio_loopback.rs` - Core audio capture functionality
- `audio_loopback/` directory:
  - `capture_engine.rs` - WASAPI capture implementation
  - `device_enumerator.rs` - Audio device discovery
  - `audio_processor.rs` - Audio data processing
  - `quality_filter.rs` - Audio quality validation
  - `settings.rs` - Configuration management
  - `types.rs` - Type definitions

### Enhanced Components
- `SettingsPanel.vue` - Complete UI overhaul with modern styling
- `ollama.rs` - Added conversational AI endpoint
- `speech.rs` - Enhanced with loopback audio support

## Usage

1. **Start a Conversation**
   - Click the conversation button in the control panel
   - Press "Start" to begin recording (microphone + system audio)

2. **Live AI Assistance**
   - Click the rocket icon to enable Live Response Assistant
   - AI analyzes conversation during pauses
   - Suggestions appear automatically when others speak
   - Copy suggestions with one click

3. **Audio Configuration**
   - Open Settings → Audio Loopback
   - Select your preferred audio device
   - Adjust buffer settings if needed

## Performance Considerations

- **Debounced Analysis**: Prevents excessive AI calls
- **Suggestion Limit**: Maximum 5 suggestions to prevent UI overflow
- **Smart Triggering**: Only analyzes when conversation pauses
- **Efficient Streaming**: Real-time updates without blocking

## Breaking Changes
None - All existing functionality remains intact.

## Testing
- ✅ Microphone transcription
- ✅ System audio loopback capture
- ✅ Live AI response generation
- ✅ Session persistence and loading
- ✅ UI responsiveness and animations
- ✅ Export functionality

## Future Enhancements
- [ ] Multi-language support for transcription
- [ ] Custom AI model selection per feature
- [ ] Advanced export formats (PDF, DOCX)
- [ ] Conversation analytics and insights

## Screenshots
*Note: The conversational interface features a dark theme with glass morphism effects, animated status indicators, and smooth transitions throughout.*

---

This PR represents a significant enhancement to Enteract's capabilities, transforming it into a comprehensive conversational assistant that helps users engage more effectively in real-time conversations.