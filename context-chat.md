## **Comprehensive Plan: Ollama Chat Context & Memory Management (Cursor Focus)**

**Overall Goal:** To implement a robust system for managing multiple chat conversations, ensuring persistent storage, intelligent context handling for AI interactions, and providing the foundational structure for a user-friendly chat history interface.

---

### **Key Concepts & Definitions:**

* **`ChatSession`**: A single, isolated conversation with the AI. It contains its own history, a title, and timestamps.
* **`ChatMessage`**: An individual message within a `ChatSession`, including sender (user/assistant/system), text content, type (text/image/file/screenshot), and a timestamp.
* **Token Estimation**: A heuristic method to approximate the number of "tokens" (units of text processed by an LLM) in a given string. This is crucial for managing context length.

---

### **Phase 1: Core Data Structures & Persistent Storage**

**Goal:** Establish the fundamental data model for chat sessions and enable their reliable saving to and loading from the local file system.

**Step 1.1: Define Frontend `ChatSession` and `ChatMessage` Interfaces**

* **File Name:** `src/types/index.ts` (or `src/types/chat.ts` if you prefer more granular typing files).
* **Action:** Modify this existing file.
* **Implementation Details:**
    * Add a new TypeScript interface named `ChatSession`.
    * This interface will include:
        * `id: string`: A unique identifier for the chat (e.g., a UUID).
        * `title: string`: A human-readable title for the chat (initially "New Chat", later auto-generated or user-defined).
        * `history: ChatMessage[]`: An array holding all `ChatMessage` objects for this session.
        * `createdAt: string`: An ISO 8601 formatted string representing the creation timestamp.
        * `updatedAt: string`: An ISO 8601 formatted string representing the last update timestamp.
        * `modelId?: string`: An optional string to store the ID of the Ollama model used for this chat (e.g., 'llama3').
    * Ensure the existing `ChatMessage` interface is compatible and includes all necessary fields (`id`, `sender`, `text`, `timestamp`, `messageType`, `file?`).
* **Dependencies/Prerequisites:** None, this is a foundational step.
* **Goal:** Create the TypeScript blueprint for how chat sessions and messages will be structured in the frontend.

**Step 1.2: Create Backend Rust Data Structures for `ChatSession` and `ChatMessage`**

* **File Name:** `src/data_store.rs` (new file).
* **Action:** Create this new file.
* **Implementation Details:**
    * Define Rust `struct`s named `ChatMessage` and `ChatSession` that mirror the TypeScript interfaces from Step 1.1.
    * Use `#[derive(Debug, Clone, Serialize, Deserialize)]` for both structs to enable easy conversion to/from JSON.
    * Ensure field names match the JSON structure (e.g., use `#[serde(rename = "messageType")]` for `message_type` if the TypeScript uses camelCase).
    * For timestamps, use `String` in Rust to store the ISO 8601 string, as this is easily handled by `serde`.
    * Also define `ChatMessageFile` and `FileDimensions` structs if they are part of your `ChatMessage` structure.
    * Define `SaveChatsPayload` struct containing a `Vec<ChatSession>` for saving.
    * Define `LoadChatsResponse` struct containing a `Vec<ChatSession>` for loading.
* **Dependencies/Prerequisites:** `serde`, `serde_json`, `chrono` (for timestamps, though `String` is used for storage, `chrono` might be useful for generating the initial timestamps).
* **Goal:** Create the Rust blueprint for how chat sessions and messages will be structured for backend processing and storage.

**Step 1.3: Implement Rust Backend Commands for Saving and Loading Chat Sessions**

* **File Name:** `src/data_store.rs` (continue modifying this file).
* **Action:** Add new Tauri commands.
* **Implementation Details:**
    * **`get_chats_file_path(app_handle: &AppHandle) -> Result<PathBuf, String>` function:**
        * This private helper function will determine the full path to the JSON file where chat sessions will be stored.
        * It should use `app_handle.path_resolver().app_data_dir()` to get a platform-agnostic application data directory.
        * It must ensure this directory exists, creating it if necessary.
        * The file name should be consistent (e.g., `"user_chat_sessions.json"`).
    * **`save_chat_sessions(app_handle: AppHandle, payload: SaveChatsPayload) -> Result<(), String>` command:**
        * Annotate with `#[tauri::command]`.
        * Takes `AppHandle` and the `SaveChatsPayload` (which contains `Vec<ChatSession>`).
        * Calls `get_chats_file_path` to get the storage location.
        * Serializes the `Vec<ChatSession>` into a pretty-printed JSON string using `serde_json::to_string_pretty`.
        * Writes this JSON string to the file using `std::fs::write`.
        * Returns `Ok(())` on success, `Err(String)` on failure.
    * **`load_chat_sessions(app_handle: AppHandle) -> Result<LoadChatsResponse, String>` command:**
        * Annotate with `#[tauri::command]`.
        * Takes `AppHandle`.
        * Calls `get_chats_file_path`.
        * Checks if the file exists; if not, returns `Ok(LoadChatsResponse { chats: Vec::new() })`.
        * Reads the file content into a string using `std::fs::read_to_string`.
        * Deserializes the JSON string back into a `Vec<ChatSession>` using `serde_json::from_str`.
        * Returns `Ok(LoadChatsResponse { chats })` on success, `Err(String)` on failure.
* **Dependencies/Prerequisites:** `tauri`, `std::fs`, `std::path::PathBuf`, `serde_json`.
* **Goal:** Create the backend logic for reading and writing chat session data to disk.

**Step 1.4: Integrate Rust Commands into `src/lib.rs`**

* **File Name:** `src/lib.rs`.
* **Action:** Modify this existing file.
* **Implementation Details:**
    * Add `mod data_store;` at the top to declare your new module.
    * Add `use data_store::{save_chat_sessions, load_chat_sessions};` to bring the commands into scope.
    * Within the `tauri::generate_handler![]` macro, add `save_chat_sessions` and `load_chat_sessions` to the list of exposed commands.
* **Dependencies/Prerequisites:** Steps 1.2 and 1.3 completed.
* **Goal:** Make the new backend persistence commands accessible from the frontend.

**Step 1.5: Update Frontend `useChatManagement.ts` for Multi-Session Management and Persistence**

* **File Name:** `src/composables/useChatManagement.ts`.
* **Action:** Modify this existing file.
* **Implementation Details:**
    * **Imports:** Import `v4 as uuidv4` from the `uuid` library (install `uuid` and `@types/uuid` if not already done via `npm install uuid @types/uuid`). Import `ChatSession` and `ChatMessage` from your `types` file.
    * **State Variables:**
        * Replace `chatHistory = ref<ChatMessage[]>([])` with:
            * `chatSessions = ref<ChatSession[]>([])`: A reactive array to hold *all* chat sessions.
            * `currentChatId = ref<string | null>(null)`: A reactive string to hold the ID of the currently active chat session.
        * Add a `computed` property `currentChatHistory` that dynamically returns the `history` array of the `ChatSession` identified by `currentChatId`.
    * **Persistence Logic:**
        * Define a constant `CHATS_STORAGE_KEY` (e.g., `'user_chat_sessions.json'`).
        * **`saveAllChats()` function:** An `async` function that invokes the `save_chat_sessions` Tauri command, passing `chatSessions.value` as the payload. Include error handling.
        * **Debounced Saving:** Implement a `debounce` utility function (e.g., 1000ms delay). Wrap `saveAllChats` with this debounce function.
        * **`watch` for `chatSessions`:** Add a deep watch (`{ deep: true }`) on `chatSessions.value` to automatically trigger the debounced `saveAllChats` whenever any chat session or its history changes.
        * **`loadAllChats()` function:** An `async` function that invokes the `load_chat_sessions` Tauri command.
            * On successful load, populate `chatSessions.value` with the returned data.
            * Set `currentChatId.value` to the ID of the most recently updated chat, or the first chat if no update order is clear.
            * If no chats are loaded (empty array), call `createNewChat()` to ensure a fresh session is available.
            * Include error handling.
    * **Chat Session Management Functions:**
        * **`createNewChat(initialMessage?: ChatMessage)`:**
            * Generates a new UUID for the `id`.
            * Sets an initial `title` (e.g., "New Chat").
            * Initializes `history` (optionally with an `initialMessage`).
            * Sets `createdAt` and `updatedAt` to the current time (ISO string).
            * Adds the new `ChatSession` to the beginning of `chatSessions.value`.
            * Sets `currentChatId.value` to the new chat's ID.
        * **`switchChat(chatId: string)`:**
            * Updates `currentChatId.value` to the provided `chatId`.
            * Includes a check to ensure the `chatId` exists in `chatSessions.value`.
            * Triggers `scrollChatToBottom()` after a short delay to ensure the view adjusts.
        * **`deleteChat(chatId: string)`:**
            * Prompts the user for confirmation.
            * Filters `chatSessions.value` to remove the specified chat.
            * If the deleted chat was the `currentChatId`, switch to the next available chat (e.g., the first one in the list) or call `createNewChat()` if no other chats remain.
            * Immediately trigger `saveAllChats` (not debounced) after deletion.
        * **`renameChat(chatId: string, newTitle: string)`:**
            * Finds the `ChatSession` by `chatId` and updates its `title` and `updatedAt` timestamp.
        * **`clearChat()`:**
            * Finds the `currentChatSession` and clears its `history` array.
            * Updates `updatedAt` timestamp.
    * **Message Sending & File Handling:**
        * Modify `sendMessage`, `handleFileChange`, `takeScreenshotAndAnalyze`, and `processClipboardImage` functions to:
            * Before adding any user message or file, check if `currentChatId.value` is null or if `currentChatHistory.value` is empty. If so, call `createNewChat()` to ensure a session is active.
            * Instead of directly pushing to `chatHistory`, use a helper like `addMessageToCurrentChat(message: ChatMessage)` which finds the `currentChatSession` and pushes the message to its `history` array.
            * Ensure `updatedAt` is updated on the `currentChatSession` whenever a message is added.
            * **Auto-Titling:** If the first message added to a *new* chat is a user message, automatically set the chat's title to a truncated version of that message (e.g., first 50 characters).
    * **Initialization:** Call `loadAllChats()` within `onMounted` to load sessions when the app starts.
    * **Return Values:** Export the new `chatSessions`, `currentChatId`, `createNewChat`, `switchChat`, `deleteChat`, `renameChat`, and `clearChat` functions from the composable, in addition to the existing ones.
* **Dependencies/Prerequisites:** `uuid` library installed. Steps 1.1, 1.2, 1.3, 1.4 completed.
* **Goal:** Enable the frontend to manage multiple chat sessions, persist them across app restarts, and correctly add messages to the active session.

**Success Criteria for Phase 1:**
* You can launch the application.
* A new, empty chat session is automatically created if no previous sessions exist.
* You can type messages, and they appear in the chat.
* You can close and reopen the application, and previous chat messages are loaded and displayed.
* You can manually verify the `user_chat_sessions.json` file exists in your app's data directory and contains the chat data.
* You can manually create new chats (if a UI element is added for it) and switch between them (if a temporary function is exposed for testing).
* Deleting a chat switches to another or creates a new one.

---

### **Phase 2: Context Handling & Intelligent Truncation**

**Goal:** Implement logic to estimate token counts and intelligently truncate chat history to fit within the LLM's context window (e.g., 4000 tokens) while preserving conversational flow.

**Step 2.1: Implement Token Estimation Utility**

* **File Name:** `src/composables/useChatManagement.ts` (or a new `src/utils/tokenEstimator.ts`).
* **Action:** Add a new function.
* **Implementation Details:**
    * **`estimateTokens(text: string): number` function:**
        * Takes a `string` as input.
        * Returns a `number` representing the estimated token count.
        * Implement the heuristic: `return Math.ceil(text.length / 4);` (as per the prompt's suggestion of ~4 characters per token).
* **Dependencies/Prerequisites:** None.
* **Goal:** Provide a reliable way to estimate the token length of any text.

**Step 2.2: Implement Context Truncation Logic**

* **File Name:** `src/composables/useChatManagement.ts`.
* **Action:** Add a new helper function.
* **Implementation Details:**
    * **`getLimitedContext(history: ChatMessage[], maxTokens: number): { role: string; content: string }[]` function:**
        * Takes the full `history` array of the current `ChatSession` and the `maxTokens` limit (e.g., 4000).
        * Returns an array of objects in the format `{ role: string; content: string }` suitable for the Ollama API.
        * **Logic:**
            1.  Initialize an empty `context` array and `currentTokens = 0`.
            2.  **Identify System Messages:** Filter out any "system" messages from the `history` and add them to the *beginning* of the `context` array. Add their estimated tokens to `currentTokens`. (System messages should always be preserved).
            3.  **Iterate Backwards:** Loop through the `history` array from the *most recent* message backwards.
            4.  For each message:
                * Estimate its token count using `estimateTokens()`.
                * If adding this message would exceed `maxTokens` (considering `currentTokens`), then stop.
                * Otherwise, add the message (converted to `{ role, content }` format) to the *beginning* of a temporary list (or `unshift` into the `context` array if performance allows for smaller histories).
                * Add the message's tokens to `currentTokens`.
            5.  After the loop, ensure the final `context` array is in chronological order (oldest messages first, then newer messages).
            6.  Consider a placeholder message if truncation occurs (e.g., `"... (history truncated) ..."`). This placeholder should also consume tokens.
* **Dependencies/Prerequisites:** Step 2.1 completed.
* **Goal:** Create a function that can intelligently select the most relevant recent messages from a chat's history, respecting a maximum token limit.

**Step 2.3: Integrate Truncation into `sendMessage` Function**

* **File Name:** `src/composables/useChatManagement.ts`.
* **Action:** Modify the `sendMessage` function.
* **Implementation Details:**
    * Inside `sendMessage`, before making the `invoke('generate_ollama_response_stream', ...)` call:
        * Call `getLimitedContext(currentChatHistory.value, 4000)` (or your configured limit) to get the truncated context.
        * Pass this *truncated context* to the `context` parameter of the `generate_ollama_response_stream` command.
        * Ensure the `context` parameter in the `invoke` call is correctly structured as an array of `{ role: string; content: string }` objects.
* **Dependencies/Prerequisites:** Steps 2.1 and 2.2 completed.
* **Goal:** Ensure that the AI model receives an optimized and appropriately sized context for its responses, preventing token limit errors and maintaining conversational coherence.

**Success Criteria for Phase 2:**
* The application continues to function normally.
* (Manual Test) You can engage in very long conversations.
* (Debug Test) By logging the `context` array sent to the `generate_ollama_response_stream` command, you can observe that older messages are being truncated when the total token count approaches the limit (e.g., 4000 tokens).
* The AI's responses remain coherent even in long conversations, indicating effective context management.

---

### **Phase 3: Frontend UI Integration (Conceptual)**

**Goal:** Provide the necessary user interface elements to allow users to manage their chat sessions, switch between them, and understand context handling.

**Step 3.1: Chat List Display**

* **File Name:** `src/App.vue` or a new component like `src/components/ChatSidebar.vue`.
* **Action:** Create or modify a Vue component.
* **Implementation Details:**
    * Use `chatSessions` from `useChatManagement` to display a list of chats.
    * Each list item should show `chat.title` and perhaps a truncated `updatedAt` timestamp.
    * Visually highlight the `currentChatId`.
* **Dependencies/Prerequisites:** Phase 1 completed.
* **Goal:** Allow users to see all their saved chat conversations.

**Step 3.2: New Chat Button**

* **File Name:** `src/App.vue` or `src/components/ChatControls.vue`.
* **Action:** Add a button.
* **Implementation Details:**
    * Add a button (e.g., "New Chat" or a `+` icon).
    * Attach a click handler that calls `createNewChat()` from `useChatManagement`.
* **Dependencies/Prerequisites:** Phase 1 completed.
* **Goal:** Provide an easy way for users to start fresh conversations.

**Step 3.3: Chat Switching**

* **File Name:** `src/components/ChatSidebar.vue` (or wherever the chat list is displayed).
* **Action:** Make chat list items interactive.
* **Implementation Details:**
    * Add a click handler to each chat list item that calls `switchChat(chat.id)` from `useChatManagement`.
* **Dependencies/Prerequisites:** Phase 1 completed.
* **Goal:** Allow users to easily navigate and resume previous conversations.

**Step 3.4: Chat Renaming and Deletion**

* **File Name:** `src/components/ChatSidebar.vue` (or next to each chat item).
* **Action:** Add controls for each chat.
* **Implementation Details:**
    * For each chat item in the list, add:
        * A "Rename" button/icon: On click, allow the user to input a new title (e.g., using a modal or inline input field) and then call `renameChat(chat.id, newTitle)`.
        * A "Delete" button/icon: On click, call `deleteChat(chat.id)`.
* **Dependencies/Prerequisites:** Phase 1 completed.
* **Goal:** Give users control over managing their chat history.

**Step 3.5: Context Truncation Indicator (Optional but Recommended)**

* **File Name:** `src/App.vue` or `src/components/ChatWindow.vue`.
* **Action:** Add a visual indicator.
* **Implementation Details:**
    * You could expose a computed property from `useChatManagement` (e.g., `isContextTruncated: computed(() => /* logic based on token count */)`) that indicates if the current chat's context is being truncated for the AI.
    * Display a small, subtle message or icon in the chat window (e.g., "History truncated for AI context") when `isContextTruncated` is true.
* **Dependencies/Prerequisites:** Phase 2 completed.
* **Goal:** Inform the user when their full conversation history is not being sent to the AI due to length limits.