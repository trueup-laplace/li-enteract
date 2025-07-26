import { ref } from 'vue'

interface OllamaModel {
  name: string
  modified_at: string
  size: number
  digest: string
  details?: {
    format: string
    family: string
    parameter_size: string
    quantization_level: string
  }
}

interface OllamaStatus {
  status: string
  version?: string
}

interface OllamaCache {
  models: OllamaModel[]
  status: OllamaStatus
  lastFetch: number
  selectedModel: string | null
}

// Cache duration in milliseconds (5 minutes)
const CACHE_DURATION = 5 * 60 * 1000

// Global cache state
const cache = ref<OllamaCache>({
  models: [],
  status: { status: 'checking' },
  lastFetch: 0,
  selectedModel: null
})

export const useOllamaCache = () => {
  const isCacheValid = () => {
    return Date.now() - cache.value.lastFetch < CACHE_DURATION
  }

  const setModels = (models: OllamaModel[]) => {
    cache.value.models = models
    cache.value.lastFetch = Date.now()
  }

  const setStatus = (status: OllamaStatus) => {
    cache.value.status = status
  }

  const setSelectedModel = (model: string | null) => {
    cache.value.selectedModel = model
  }

  const getCache = () => cache.value

  const clearCache = () => {
    cache.value.lastFetch = 0
  }

  return {
    isCacheValid,
    setModels,
    setStatus,
    setSelectedModel,
    getCache,
    clearCache
  }
}