<script setup lang="ts">
import { type PropType } from 'vue'
import { ArrowsPointingOutIcon, CpuChipIcon } from '@heroicons/vue/24/outline'

interface SystemInfoGpu {
  name: string
  vendor: string
  driver_version?: string
  memory_mb?: number
  temperature_celsius?: number
  utilization_percent?: number
}

interface SystemInfo {
  gpus: SystemInfoGpu[]
  cpu_name: string
  memory_gb: number
  os: string
}

const props = defineProps({
  generalSettings: { type: Object as PropType<any>, required: true },
  transparency: { type: Object as PropType<any>, required: true },
  handleTransparencyLevelChange: { type: Function as PropType<() => void>, required: true },
  fetchSystemInfo: { type: Function as PropType<() => Promise<void> | void>, required: true },
  systemInfo: { type: Object as PropType<SystemInfo | null>, required: false, default: null },
  isLoadingSystemInfo: { type: Boolean, required: true },
  systemInfoError: { type: String as PropType<string | null>, required: false, default: null },
  formatGpuMemory: { type: Function as PropType<(mb?: number) => string>, required: true },
  setGeneralSetting: { type: Function as PropType<(key: string, value: any) => void>, required: true }
})
</script>

<template>
  <div class="settings-section">
    <div class="section-header">
      <h2 class="section-title">General Settings</h2>
      <p class="section-description">
        Customize Enteract's behavior and appearance to match your workflow preferences.
      </p>
    </div>

    <div class="settings-group">
      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="generalSettings.autoStartOllama"
            @change="(e: Event) => setGeneralSetting('autoStartOllama', (e.target as HTMLInputElement).checked)"
            class="setting-checkbox"
          >
          <span class="text-white/90">Auto-start Ollama</span>
        </label>
        <p class="text-white/60 text-xs mt-1">Automatically start Ollama when the app launches</p>
      </div>

      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="generalSettings.enableNotifications"
            @change="(e: Event) => setGeneralSetting('enableNotifications', (e.target as HTMLInputElement).checked)"
            class="setting-checkbox"
          >
          <span class="text-white/90">Enable Notifications</span>
        </label>
        <p class="text-white/60 text-xs mt-1">Show desktop notifications for important events</p>
      </div>

      <div class="setting-item">
        <label class="setting-label-full">
          <span class="text-white/90">Theme</span>
          <select :value="generalSettings.theme" @change="(e: Event) => setGeneralSetting('theme', (e.target as HTMLSelectElement).value)" class="setting-select">
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="auto">Auto</option>
          </select>
        </label>
      </div>

      <div class="setting-item">
        <label class="setting-label-full">
          <span class="text-white/90">Log Level</span>
          <select :value="generalSettings.logLevel" @change="(e: Event) => setGeneralSetting('logLevel', (e.target as HTMLSelectElement).value)" class="setting-select">
            <option value="debug">Debug</option>
            <option value="info">Info</option>
            <option value="warn">Warning</option>
            <option value="error">Error</option>
          </select>
        </label>
      </div>

      <div class="setting-separator"></div>

      <h4 class="text-white/80 text-sm font-medium mb-3">Startup & Window</h4>

      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="generalSettings.startWithSystem"
            @change="(e: Event) => setGeneralSetting('startWithSystem', (e.target as HTMLInputElement).checked)"
            class="setting-checkbox"
          >
          <span class="text-white/90">Start with System</span>
        </label>
        <p class="text-white/60 text-xs mt-1">Launch Enteract when your computer starts</p>
      </div>

      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="generalSettings.startMinimized"
            @change="(e: Event) => setGeneralSetting('startMinimized', (e.target as HTMLInputElement).checked)"
            class="setting-checkbox"
          >
          <span class="text-white/90">Start Minimized</span>
        </label>
        <p class="text-white/60 text-xs mt-1">Start in the system tray instead of showing the window</p>
      </div>

      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="generalSettings.saveWindowPosition"
            @change="(e: Event) => setGeneralSetting('saveWindowPosition', (e.target as HTMLInputElement).checked)"
            class="setting-checkbox"
          >
          <span class="text-white/90">Remember Window Position</span>
        </label>
        <p class="text-white/60 text-xs mt-1">Restore window size and position on startup</p>
      </div>

      <div class="setting-separator"></div>

      <h4 class="text-white/80 text-sm font-medium mb-3">Transcription</h4>

      <div class="setting-item">
        <label class="setting-label-full">
          <span class="text-white/90">Default Language</span>
          <select :value="generalSettings.transcriptionLanguage" @change="(e: Event) => setGeneralSetting('transcriptionLanguage', (e.target as HTMLSelectElement).value)" class="setting-select">
            <option value="en">English</option>
            <option value="es">Spanish</option>
            <option value="fr">French</option>
            <option value="de">German</option>
            <option value="it">Italian</option>
            <option value="pt">Portuguese</option>
            <option value="ru">Russian</option>
            <option value="ja">Japanese</option>
            <option value="ko">Korean</option>
            <option value="zh">Chinese</option>
          </select>
        </label>
      </div>

      <div class="setting-item">
        <label class="setting-label-full">
          <span class="text-white/90">Microphone Whisper Model</span>
          <select :value="generalSettings.microphoneWhisperModel" @change="(e: Event) => setGeneralSetting('microphoneWhisperModel', (e.target as HTMLSelectElement).value)" class="setting-select">
            <option value="tiny">Tiny (Fastest, Good accuracy)</option>
            <option value="base">Base (Balanced speed/accuracy)</option>
            <option value="small">Small (Best accuracy, Slower)</option>
          </select>
        </label>
        <p class="text-white/60 text-xs mt-1">Model used for microphone transcription. Tiny is recommended for real-time performance.</p>
      </div>

      <div class="setting-item">
        <label class="setting-label-full">
          <span class="text-white/90">System Audio Whisper Model</span>
          <select :value="generalSettings.loopbackWhisperModel" @change="(e: Event) => setGeneralSetting('loopbackWhisperModel', (e.target as HTMLSelectElement).value)" class="setting-select">
            <option value="tiny">Tiny (Fastest, Good accuracy)</option>
            <option value="base">Base (Balanced speed/accuracy)</option>
            <option value="small">Small (Best accuracy, Slower)</option>
          </select>
        </label>
        <p class="text-white/60 text-xs mt-1">Model used for system audio loopback transcription. Base is recommended for better accuracy with recorded audio.</p>
      </div>

      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="generalSettings.enableKeyboardShortcuts"
            @change="(e: Event) => setGeneralSetting('enableKeyboardShortcuts', (e.target as HTMLInputElement).checked)"
            class="setting-checkbox"
          >
          <span class="text-white/90">Enable Keyboard Shortcuts</span>
        </label>
        <p class="text-white/60 text-xs mt-1">Use global keyboard shortcuts for quick actions</p>
      </div>

      <div class="setting-separator"></div>

      <h4 class="text-white/80 text-sm font-medium mb-3">Auto-save</h4>

      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="generalSettings.enableAutoSave"
            @change="(e: Event) => setGeneralSetting('enableAutoSave', (e.target as HTMLInputElement).checked)"
            class="setting-checkbox"
          >
          <span class="text-white/90">Enable Auto-save</span>
        </label>
        <p class="text-white/60 text-xs mt-1">Automatically save chat sessions</p>
      </div>

      <div class="setting-item" v-if="generalSettings.enableAutoSave">
        <label class="setting-label-full">
          <span class="text-white/90">Auto-save Interval: {{ generalSettings.autoSaveInterval }} minutes</span>
          <input 
            type="range" 
            :value="generalSettings.autoSaveInterval"
            @input="(e: Event) => setGeneralSetting('autoSaveInterval', Number((e.target as HTMLInputElement).value))"
            min="1"
            max="30"
            step="1"
            class="setting-range"
          >
        </label>
      </div>

      <div class="setting-separator"></div>

      <h4 class="text-white/80 text-sm font-medium mb-3">Transparency</h4>

      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="generalSettings.enableTransparency"
            @change="(e: Event) => setGeneralSetting('enableTransparency', (e.target as HTMLInputElement).checked)"
            class="setting-checkbox"
          >
          <span class="text-white/90">Enable Transparency</span>
        </label>
        <p class="text-white/60 text-xs mt-1">Allow window transparency effects</p>
      </div>

      <div class="setting-item" v-if="generalSettings.enableTransparency">
        <label class="setting-label-full">
          <span class="text-white/90">Default Transparency Level: {{ Math.round(generalSettings.defaultTransparencyLevel * 100) }}%</span>
          <input 
            type="range" 
            :value="generalSettings.defaultTransparencyLevel"
            @input="(e: Event) => { setGeneralSetting('defaultTransparencyLevel', Number((e.target as HTMLInputElement).value)); handleTransparencyLevelChange(); }"
            min="0.1"
            max="1.0"
            step="0.1"
            class="setting-range"
          >
        </label>
        <p class="text-white/60 text-xs mt-1">Default opacity when transparency is enabled</p>
      </div>

      <div class="setting-item" v-if="generalSettings.enableTransparency">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="generalSettings.autoRestoreOnError"
            @change="(e: Event) => setGeneralSetting('autoRestoreOnError', (e.target as HTMLInputElement).checked)"
            class="setting-checkbox"
          >
          <span class="text-white/90">Auto-restore on Error</span>
        </label>
        <p class="text-white/60 text-xs mt-1">Automatically restore window if transparency fails</p>
      </div>

      <div v-if="generalSettings.enableTransparency" class="transparency-status-section">
        <div class="flex items-center justify-between p-3 rounded-lg bg-white/5 border border-white/10">
          <div class="flex items-center gap-2">
            <div class="w-2 h-2 rounded-full" :class="{
              'bg-green-400': transparency.isVisible.value && !transparency.isClickThrough.value,
              'bg-yellow-400': transparency.isClickThrough.value,
              'bg-red-400': !transparency.isVisible.value
            }"></div>
            <span class="text-white/80 text-sm">Current Status: {{ transparency.getVisibilityStatus() }}</span>
          </div>
          <div class="text-white/60 text-xs">
            {{ transparency.getTransparencyPercentage() }}% opacity
          </div>
        </div>

        <div class="flex gap-2 mt-3">
          <button
            @click="() => { setGeneralSetting('defaultTransparencyLevel', 1.0); handleTransparencyLevelChange(); }"
            class="px-3 py-1.5 text-xs rounded-lg bg-white/10 hover:bg-white/20 text-white/80 hover:text-white transition-colors"
          >
            Solid
          </button>
          <button
            @click="() => { setGeneralSetting('defaultTransparencyLevel', 0.7); handleTransparencyLevelChange(); }"
            class="px-3 py-1.5 text-xs rounded-lg bg-white/10 hover:bg-white/20 text-white/80 hover:text-white transition-colors"
          >
            Semi
          </button>
          <button
            @click="() => { setGeneralSetting('defaultTransparencyLevel', 0.3); handleTransparencyLevelChange(); }"
            class="px-3 py-1.5 text-xs rounded-lg bg-white/10 hover:bg-white/20 text-white/80 hover:text-white transition-colors"
          >
            Ghost
          </button>
          <button
            @click="() => { setGeneralSetting('defaultTransparencyLevel', 1.0); handleTransparencyLevelChange(); }"
            class="px-3 py-1.5 text-xs rounded-lg bg-red-500/20 hover:bg-red-500/30 text-red-400 hover:text-red-300 transition-colors"
          >
            Restore
          </button>
        </div>
      </div>

      <div class="setting-separator"></div>

      <h4 class="text-white/80 text-sm font-medium mb-3">System Information</h4>

      <div v-if="isLoadingSystemInfo" class="text-white/60 text-sm">
        Loading system information...
      </div>

      <div v-else-if="systemInfoError" class="text-red-400 text-sm">
        Failed to load system info: {{ systemInfoError }}
      </div>

      <div v-else-if="systemInfo" class="system-info">
        <div class="info-item">
          <span class="info-label">OS:</span>
          <span class="info-value">{{ systemInfo.os }}</span>
        </div>

        <div class="info-item">
          <span class="info-label">CPU:</span>
          <span class="info-value">{{ systemInfo.cpu_name }}</span>
        </div>

        <div class="info-item">
          <span class="info-label">Memory:</span>
          <span class="info-value">{{ systemInfo.memory_gb.toFixed(1) }} GB</span>
        </div>

        <div v-if="systemInfo.gpus.length > 0" class="gpu-section">
          <div class="info-item">
            <span class="info-label">GPU{{ systemInfo.gpus.length > 1 ? 's' : '' }}:</span>
          </div>

          <div v-for="(gpu, index) in systemInfo.gpus" :key="index" class="gpu-info">
            <div class="gpu-header">
              <CpuChipIcon class="w-4 h-4 text-white/60" />
              <span class="gpu-name">{{ gpu.name }}</span>
              <span class="gpu-vendor">({{ gpu.vendor }})</span>
            </div>

            <div class="gpu-details">
              <div v-if="gpu.memory_mb" class="gpu-detail">
                <span class="detail-label">Memory:</span>
                <span class="detail-value">{{ formatGpuMemory(gpu.memory_mb) }}</span>
              </div>

              <div v-if="gpu.driver_version" class="gpu-detail">
                <span class="detail-label">Driver:</span>
                <span class="detail-value">{{ gpu.driver_version }}</span>
              </div>

              <div v-if="gpu.temperature_celsius !== undefined" class="gpu-detail">
                <span class="detail-label">Temperature:</span>
                <span class="detail-value">{{ gpu.temperature_celsius }}°C</span>
              </div>

              <div v-if="gpu.utilization_percent !== undefined" class="gpu-detail">
                <span class="detail-label">Usage:</span>
                <span class="detail-value">{{ gpu.utilization_percent }}%</span>
              </div>
            </div>
          </div>
        </div>

        <div v-else class="info-item">
          <span class="info-label">GPU:</span>
          <span class="info-value text-white/60">No dedicated GPU detected</span>
        </div>

        <button @click="fetchSystemInfo" class="refresh-button mt-3">
          <ArrowsPointingOutIcon class="w-4 h-4" />
          Refresh System Info
        </button>
      </div>
    </div>
  </div>
</template>


