<script setup lang="ts">
import { type PropType } from 'vue'
import { ArrowsPointingOutIcon } from '@heroicons/vue/24/outline'

interface AudioLoopbackDevice {
  id: string
  name: string
  is_default: boolean
  sample_rate: number
  channels: number
  format: string
  device_type: 'Render' | 'Capture'
  loopback_method: 'RenderLoopback' | 'CaptureDevice' | 'StereoMix'
}

interface AudioDeviceSettings {
  selectedLoopbackDevice: string | null
  loopbackEnabled: boolean
  bufferSize: number
  sampleRate: number
}

defineProps({
  audioDevices: { type: Array as PropType<AudioLoopbackDevice[]>, required: true },
  isLoadingAudioDevices: { type: Boolean, required: true },
  audioDevicesError: { type: String as PropType<string | null>, required: false, default: null },
  testingDeviceId: { type: String as PropType<string | null>, required: false, default: null },
  audioSettings: { type: Object as PropType<AudioDeviceSettings>, required: true },
  enumerateAudioDevices: { type: Function as PropType<() => Promise<void> | void>, required: true },
  clearAudioError: { type: Function as PropType<() => void>, required: true },
  selectAudioDevice: { type: Function as PropType<(id: string) => Promise<void> | void>, required: true },
  getDeviceIcon: { type: Function as PropType<(device: AudioLoopbackDevice) => any>, required: true },
  getDeviceMethodBadge: { type: Function as PropType<(method: string) => { text: string; class: string }>, required: true },
  setAudioLoopbackEnabled: { type: Function as PropType<(v: boolean) => void>, required: true },
  setBufferSize: { type: Function as PropType<(v: number) => void>, required: true },
  setSampleRate: { type: Function as PropType<(v: number) => void>, required: true }
})
</script>

<template>
  <div class="settings-section">
    <div class="section-header">
      <h2 class="section-title">Audio Loopback</h2>
      <p class="section-description">
        Configure audio loopback to capture system audio for transcription. Perfect for meetings, videos, and any audio playing on your computer.
      </p>
    </div>

    <div class="audio-settings-header">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-white/90 font-medium">Audio Loopback Devices</h3>
        <button 
          @click="enumerateAudioDevices" 
          :disabled="isLoadingAudioDevices"
          class="refresh-btn"
          title="Refresh Audio Devices"
        >
          <ArrowsPointingOutIcon class="w-4 h-4" :class="{ 'animate-spin': isLoadingAudioDevices }" />
        </button>
      </div>

      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="audioSettings.loopbackEnabled"
            @change="(e: Event) => setAudioLoopbackEnabled((e.target as HTMLInputElement).checked)"
            class="setting-checkbox"
          >
          <span class="text-white/90">Enable Audio Loopback</span>
        </label>
        <p class="text-white/60 text-xs mt-1">Capture system audio for conversational interface</p>
      </div>
    </div>

    <div v-if="audioDevicesError" class="error-message">
      <span class="text-red-400 text-sm">{{ audioDevicesError }}</span>
      <button @click="clearAudioError" class="ml-2 text-white/60 hover:text-white">×</button>
    </div>

    <div v-if="isLoadingAudioDevices" class="loading-state">
      <div class="animate-pulse text-white/60">Scanning WASAPI audio devices...</div>
    </div>

    <div v-else-if="audioDevices.length > 0" class="audio-devices-list">
      <div v-for="device in audioDevices" :key="device.id" class="audio-device-item">
        <div class="device-info">
          <div class="device-header">
            <component :is="getDeviceIcon(device)" class="w-4 h-4 text-white/80" />
            <div class="device-name">{{ device.name }}</div>
            <div v-if="device.is_default" class="default-badge">Default</div>
          </div>

          <div class="device-details">
            <span class="device-spec">{{ device.sample_rate }} Hz</span>
            <span class="device-spec">{{ device.channels }} ch</span>
            <span class="device-spec">{{ device.format }}</span>

            <span 
              class="method-badge"
              :class="getDeviceMethodBadge(device.loopback_method).class"
            >
              {{ getDeviceMethodBadge(device.loopback_method).text }}
            </span>
          </div>
        </div>

        <div class="device-actions">
          <button
            @click="() => { selectAudioDevice(device.id) }"
            :class="{ 'active': audioSettings.selectedLoopbackDevice === device.id }"
            class="select-btn"
            title="Select Device"
            :disabled="isLoadingAudioDevices || testingDeviceId !== null"
            type="button"
          >
            <span v-if="testingDeviceId === device.id" class="animate-spin">⟳</span>
            <span v-else>{{ audioSettings.selectedLoopbackDevice === device.id ? '✓' : '○' }}</span>
          </button>
        </div>
      </div>
    </div>

    <div v-else class="no-devices">
      <p class="text-white/60 text-sm">No loopback devices found</p>
      <p class="text-white/40 text-xs mt-1">Try enabling Stereo Mix in Windows Sound settings</p>
    </div>

    <div v-if="audioSettings.loopbackEnabled" class="audio-buffer-settings">
      <h4 class="text-white/80 text-sm font-medium mb-3">Buffer Settings</h4>

      <div class="setting-item">
        <label class="setting-label-full">
          <span class="text-white/90">Buffer Size: {{ audioSettings.bufferSize }} samples</span>
          <input 
            type="range" 
            :value="audioSettings.bufferSize"
            @input="(e: Event) => setBufferSize(Number((e.target as HTMLInputElement).value))"
            min="1024"
            max="8192"
            step="1024"
            class="setting-range"
          >
        </label>
      </div>

      <div class="setting-item">
        <label class="setting-label-full">
          <span class="text-white/90">Sample Rate: {{ audioSettings.sampleRate }} Hz</span>
          <select :value="audioSettings.sampleRate" @change="(e: Event) => setSampleRate(Number((e.target as HTMLSelectElement).value))" class="setting-select">
            <option :value="16000">16000 Hz (Whisper)</option>
            <option :value="44100">44100 Hz (CD Quality)</option>
            <option :value="48000">48000 Hz (Studio)</option>
          </select>
        </label>
      </div>
    </div>
  </div>
</template>


