// fileService.ts - Handles file upload functionality
import { SessionManager } from './sessionManager'

let messageIdCounter = 1

export class FileService {
  private static scrollChatToBottom: () => void

  static init(scrollCallback: () => void) {
    FileService.scrollChatToBottom = scrollCallback
  }

  static triggerFileUpload(fileInput: HTMLInputElement | undefined) {
    fileInput?.click()
  }

  static async handleFileUpload(event: Event, showChatWindow: any) {
    const input = event.target as HTMLInputElement
    const files = input.files
    if (files) {
      // Auto-open chat window if not already open
      if (!showChatWindow.value) {
        showChatWindow.value = true
        console.log('💬 Chat window auto-opened for file upload')
      }

      for (let i = 0; i < files.length; i++) {
        const file = files[i]
        try {
          // Enhanced file upload indication
          console.log('📁 File selected:', file.name, file.type, file.size)
          
          // Add file upload message to current chat
          SessionManager.addMessageToCurrentChat({
            id: messageIdCounter++,
            sender: 'system',
            text: `📁 File uploaded: **${file.name}** (${file.type}, ${(file.size / 1024).toFixed(1)} KB)`,
            timestamp: new Date(),
            messageType: 'text'
          })
          
          // Show upload success feedback
          setTimeout(() => {
            SessionManager.addMessageToCurrentChat({
              id: messageIdCounter++,
              sender: 'system',
              text: `✅ File ready for analysis. You can now ask questions about this ${file.type.includes('image') ? 'image' : 'document'}.`,
              timestamp: new Date(),
              messageType: 'text'
            })
            
            // Auto-scroll to show the uploaded file message
            setTimeout(() => {
              FileService.scrollChatToBottom()
            }, 100)
          }, 500)
          
        } catch (error) {
          console.error('File upload error:', error)
          SessionManager.addMessageToCurrentChat({
            id: messageIdCounter++,
            sender: 'system',
            text: `❌ File upload failed: ${error}`,
            timestamp: new Date(),
            messageType: 'text'
          })
        }
      }
      
      // Auto-scroll to show files
      setTimeout(() => {
        FileService.scrollChatToBottom()
      }, 150)
    }
    // Reset input
    input.value = ''
  }
}