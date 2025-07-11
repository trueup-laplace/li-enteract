# Cursor Prompt: Ollama Chat Context Management

You are an expert developer tasked with adding chat context management to an Ollama desktop application. The system needs to remember conversations, allow users to start new chats, and provide access to chat history.

## Core Requirements

### Chat Management
- Each chat session maintains its own conversation history
- Users can start new blank chats at any time
- All previous chats are stored and searchable
- Conversations are completely isolated between different chat sessions
- Chat history survives app restarts

### Context Handling
- Maximum context length: 4000 tokens (configurable)
- When conversations get too long, intelligently truncate while preserving important messages
- Always keep system prompts and recent messages
- Estimate tokens at ~4 characters per token

### Storage & Search
- Persistent storage that survives app restarts
- Fast search through all chat content
- Automatic chat titles from first user message
- Options to rename, delete, or export chats

## Implementation Tasks

### 1. Chat Management System
Create a system to handle multiple chat sessions:
- Create new chat sessions with unique IDs
- Add messages to specific chats
- Retrieve conversation context for Ollama API calls
- Handle context truncation when conversations exceed limits
- Delete and manage chat sessions

### 2. UI Components
Add necessary interface elements:
- Button to start new chats
- Searchable list of all previous chats
- Current active chat display
- Basic chat management options (rename, delete)

### 3. Context Management
Implement smart conversation handling:
- Truncate context when approaching token limits
- Keep system prompts and recent messages as priority
- Summarize older messages when needed
- Handle very long individual messages gracefully

### 4. Storage Solution
Implement reliable data persistence:
- Save all chat sessions automatically
- Fast retrieval and search capabilities
- Data integrity and recovery mechanisms
- Support for importing/exporting chat history

### 5. Ollama Integration
Connect seamlessly with existing Ollama setup:
- Send appropriate context with each API request
- Handle streaming responses
- Maintain conversation flow
- Support for different models

## Key Features

### Chat History Management
- Automatic saving of all conversations
- Search through all chat content
- Group chats by date for organization
- Quick access to recent chats

### Context Intelligence
- Smart truncation that preserves conversation flow
- Token counting for accurate context management
- Graceful handling of edge cases
- Visual indicators when context is truncated

### User Experience
- Intuitive chat switching
- Fast search with highlighted results
- Reliable performance with large chat histories
- Error handling with clear recovery options

## Success Criteria

The implementation should provide:
1. **Seamless chat management** - Users can easily create, switch between, and manage multiple conversations
2. **Reliable context handling** - Conversations maintain context appropriately without hitting token limits
3. **Fast search and access** - Users can quickly find and access previous conversations
4. **Persistent storage** - All chat data is reliably saved and recoverable
5. **Smooth Ollama integration** - No disruption to existing chat functionality

Focus on creating a clean, reliable system that enhances the user experience without adding complexity. The goal is to make managing multiple Ollama conversations effortless and intuitive.