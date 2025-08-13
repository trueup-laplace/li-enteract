import { ref, computed } from 'vue'
import { ragService, type Document, type DocumentChunk, type RagSettings } from '../services/ragService'

export function useRagDocuments() {
  // State
  const documents = ref<Document[]>([])
  const selectedDocumentIds = ref<Set<string>>(new Set())
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const uploadProgress = ref<Map<string, number>>(new Map())
  const settings = ref<RagSettings | null>(null)
  const searchResults = ref<DocumentChunk[]>([])
  const isSearching = ref(false)

  // Computed
  const selectedDocuments = computed(() => {
    return documents.value.filter(doc => selectedDocumentIds.value.has(doc.id))
  })

  const cachedDocuments = computed(() => {
    return documents.value.filter(doc => doc.is_cached)
  })

  const totalStorageSize = computed(() => {
    return documents.value.reduce((sum, doc) => sum + doc.file_size, 0)
  })

  const totalStorageSizeMB = computed(() => {
    return totalStorageSize.value / (1024 * 1024)
  })

  // Initialize RAG system
  const initialize = async () => {
    try {
      isLoading.value = true
      error.value = null
      
      await ragService.initialize()
      await loadDocuments()
      await loadSettings()
      
      console.log('RAG system initialized with documents:', documents.value.length)
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to initialize RAG system'
      console.error('Failed to initialize RAG system:', err)
    } finally {
      isLoading.value = false
    }
  }

  // Load all documents
  const loadDocuments = async () => {
    try {
      isLoading.value = true
      error.value = null
      
      const docs = await ragService.getAllDocuments()
      documents.value = docs
      
      // Restore selected documents from localStorage
      const savedSelection = localStorage.getItem('rag_selected_documents')
      if (savedSelection) {
        const savedIds = JSON.parse(savedSelection) as string[]
        selectedDocumentIds.value = new Set(savedIds.filter(id => 
          documents.value.some(doc => doc.id === id)
        ))
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load documents'
      console.error('Failed to load documents:', err)
    } finally {
      isLoading.value = false
    }
  }

  // Load settings
  const loadSettings = async () => {
    try {
      settings.value = await ragService.getSettings()
    } catch (err) {
      console.error('Failed to load RAG settings:', err)
    }
  }

  // Upload document with progress tracking
  const uploadDocument = async (file: File): Promise<Document | null> => {
    try {
      error.value = null
      
      // Validate file
      if (settings.value) {
        const validation = ragService.validateFile(file, settings.value)
        if (!validation.valid) {
          error.value = validation.error || 'File validation failed'
          return null
        }
      }
      
      // Track upload progress
      const fileId = `${file.name}-${Date.now()}`
      uploadProgress.value.set(fileId, 0)
      
      // Simulate progress updates (real progress would come from backend)
      const progressInterval = setInterval(() => {
        const current = uploadProgress.value.get(fileId) || 0
        if (current < 90) {
          uploadProgress.value.set(fileId, current + 10)
        }
      }, 200)
      
      // Upload document
      const document = await ragService.uploadDocument(file)
      
      // Complete progress
      clearInterval(progressInterval)
      uploadProgress.value.set(fileId, 100)
      
      // Add to documents list
      documents.value.unshift(document)
      
      // Auto-select newly uploaded document
      selectedDocumentIds.value.add(document.id)
      saveSelectedDocuments()
      
      // Clear progress after delay
      setTimeout(() => {
        uploadProgress.value.delete(fileId)
      }, 2000)
      
      return document
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to upload document'
      console.error('Failed to upload document:', err)
      return null
    }
  }

  // Upload multiple documents
  const uploadDocuments = async (files: FileList | File[]): Promise<Document[]> => {
    const uploaded: Document[] = []
    
    for (const file of files) {
      const doc = await uploadDocument(file)
      if (doc) {
        uploaded.push(doc)
      }
    }
    
    return uploaded
  }

  // Delete document
  const deleteDocument = async (documentId: string) => {
    try {
      error.value = null
      
      await ragService.deleteDocument(documentId)
      
      // Remove from local state
      documents.value = documents.value.filter(doc => doc.id !== documentId)
      selectedDocumentIds.value.delete(documentId)
      saveSelectedDocuments()
      
      console.log(`Document ${documentId} deleted`)
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete document'
      console.error('Failed to delete document:', err)
    }
  }

  // Toggle document selection
  const toggleDocumentSelection = (documentId: string) => {
    if (selectedDocumentIds.value.has(documentId)) {
      selectedDocumentIds.value.delete(documentId)
    } else {
      // Check if we're at the cache limit
      if (settings.value && selectedDocumentIds.value.size >= settings.value.max_cached_documents) {
        // Remove the oldest selected document
        const oldestId = Array.from(selectedDocumentIds.value)[0]
        selectedDocumentIds.value.delete(oldestId)
      }
      selectedDocumentIds.value.add(documentId)
    }
    saveSelectedDocuments()
  }

  // Select all documents
  const selectAllDocuments = () => {
    documents.value.forEach(doc => {
      if (selectedDocumentIds.value.size < (settings.value?.max_cached_documents || 5)) {
        selectedDocumentIds.value.add(doc.id)
      }
    })
    saveSelectedDocuments()
  }

  // Clear selection
  const clearSelection = () => {
    selectedDocumentIds.value.clear()
    saveSelectedDocuments()
  }

  // Save selected documents to localStorage
  const saveSelectedDocuments = () => {
    const selectedIds = Array.from(selectedDocumentIds.value)
    localStorage.setItem('rag_selected_documents', JSON.stringify(selectedIds))
  }

  // Search documents
  const searchDocuments = async (query: string, useSelectedOnly = true) => {
    try {
      isSearching.value = true
      error.value = null
      
      const contextIds = useSelectedOnly ? Array.from(selectedDocumentIds.value) : []
      searchResults.value = await ragService.searchDocuments(query, contextIds)
      
      return searchResults.value
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Search failed'
      console.error('Document search failed:', err)
      return []
    } finally {
      isSearching.value = false
    }
  }

  // Get document by ID
  const getDocumentById = (documentId: string): Document | undefined => {
    return documents.value.find(doc => doc.id === documentId)
  }

  // Generate embeddings for a document
  const generateEmbeddings = async (documentId: string) => {
    try {
      await ragService.generateEmbeddings(documentId)
      
      // Update document cache status
      const doc = documents.value.find(d => d.id === documentId)
      if (doc) {
        doc.is_cached = true
      }
    } catch (err) {
      console.error('Failed to generate embeddings:', err)
    }
  }

  // Clear embedding cache
  const clearEmbeddingCache = async () => {
    try {
      await ragService.clearEmbeddingCache()
      
      // Update all documents cache status
      documents.value.forEach(doc => {
        doc.is_cached = false
      })
    } catch (err) {
      console.error('Failed to clear embedding cache:', err)
    }
  }

  // Update settings
  const updateSettings = async (newSettings: Partial<RagSettings>) => {
    try {
      if (!settings.value) return
      
      const updatedSettings = { ...settings.value, ...newSettings }
      await ragService.updateSettings(updatedSettings)
      settings.value = updatedSettings
    } catch (err) {
      console.error('Failed to update settings:', err)
    }
  }

  // Get storage statistics
  const getStorageStats = async () => {
    try {
      return await ragService.getStorageStats()
    } catch (err) {
      console.error('Failed to get storage stats:', err)
      return null
    }
  }

  // Format document context for AI
  const formatContextForAI = (chunks: DocumentChunk[]): string => {
    if (chunks.length === 0) return ''
    
    const grouped = chunks.reduce((acc, chunk) => {
      if (!acc[chunk.document_id]) {
        acc[chunk.document_id] = []
      }
      acc[chunk.document_id].push(chunk)
      return acc
    }, {} as Record<string, DocumentChunk[]>)
    
    let context = 'Relevant document context:\n\n'
    
    for (const [docId, docChunks] of Object.entries(grouped)) {
      const doc = getDocumentById(docId)
      if (doc) {
        context += `From "${doc.file_name}":\n`
        docChunks.forEach(chunk => {
          context += `- ${chunk.content.trim()}\n`
        })
        context += '\n'
      }
    }
    
    return context
  }

  return {
    // State
    documents,
    selectedDocumentIds,
    selectedDocuments,
    cachedDocuments,
    isLoading,
    error,
    uploadProgress,
    settings,
    searchResults,
    isSearching,
    totalStorageSize,
    totalStorageSizeMB,
    
    // Methods
    initialize,
    loadDocuments,
    loadSettings,
    uploadDocument,
    uploadDocuments,
    deleteDocument,
    toggleDocumentSelection,
    selectAllDocuments,
    clearSelection,
    searchDocuments,
    getDocumentById,
    generateEmbeddings,
    clearEmbeddingCache,
    updateSettings,
    getStorageStats,
    formatContextForAI
  }
}