export interface TranscriptionResult {
  text: string;
  isFinal: boolean;
  confidence: number;
  timestamp: number;
  source: 'web-speech' | 'whisper';
}

export interface AudioChunk {
  data: Uint8Array;
  timestamp: number;
  duration: number;
}

export interface WhisperConfig {
  modelSize: 'tiny' | 'base' | 'small' | 'medium' | 'large';
  language?: string;
  enableVAD: boolean;
  silenceThreshold: number;
  maxSegmentLength: number;
}

export interface TranscriptionSession {
  id: string;
  isActive: boolean;
  startTime: number;
  language: string;
  config: WhisperConfig;
}

export interface SpeechRecognitionConfig {
  continuous: boolean;
  interimResults: boolean;
  language: string;
  maxAlternatives: number;
}

export interface AudioStreamConfig {
  sampleRate: number;
  channels: number;
  bufferSize: number;
  mimeType: string;
}

export interface TranscriptionStatus {
  isRecording: boolean;
  isProcessing: boolean;
  hasWebSpeechSupport: boolean;
  hasWhisperModel: boolean;
  currentSession?: TranscriptionSession;
} 