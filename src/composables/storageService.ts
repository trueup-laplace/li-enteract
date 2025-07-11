// storageService.ts - Handles chat persistence
import { invoke } from '@tauri-apps/api/core'
import type { ChatSession, SaveChatsPayload, LoadChatsResponse } from '../types/chat'

export class StorageService {
  private static readonly STORAGE_KEY = 'user_chat_sessions.json'
  
  // Debounce utility
  private static debounce(func: Function, delay: number) {
    let timeoutId: number
    return (...args: any[]) => {
      clearTimeout(timeoutId)
      timeoutId = window.setTimeout(() => func.apply(null, args), delay)
    }
  }

  static async saveAllChats(chatSessions: ChatSession[]) {
    try {
      const payload: SaveChatsPayload = { chats: chatSessions }
      await invoke('save_chat_sessions', { payload })
      console.log('✅ Chat sessions saved successfully')
    } catch (error) {
      console.error('❌ Failed to save chat sessions:', error)
    }
  }

  // Debounced save function (1000ms delay)
  static debouncedSaveChats = StorageService.debounce(StorageService.saveAllChats, 1000)

  static async loadAllChats(): Promise<ChatSession[]> {
    try {
      const response: LoadChatsResponse = await invoke('load_chat_sessions')
      if (response.chats && response.chats.length > 0) {
        console.log(`✅ Loaded ${response.chats.length} chat sessions`)
        return response.chats
      }
      return []
    } catch (error) {
      console.error('❌ Failed to load chat sessions:', error)
      return []
    }
  }
}