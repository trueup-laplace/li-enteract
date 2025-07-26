### **New Phase: Enhanced Voice Interaction & Wake Word Integration**

**Goal:** To simplify voice input by consolidating manual transcription to a single mic button and introducing a "hands-free" option via the "Aubrey" wake word, ensuring all voice inputs direct to the active or most recent chat.

**Phase 1: UI Simplification (Frontend)**

* **Objective:** Remove redundant voice activation buttons and clarify the role of the mic button.
* **Key Actions:**
    1.  **Remove Wake Word Button:** Identify and remove any dedicated "wake word" or "always-on listening" button from the main chat UI.
    2.  **Mic Button Clarity:** Ensure the existing "mic" button is clearly and solely responsible for initiating transcription when clicked.

**Phase 2: Mic Button Transcription Logic (Frontend)**

* **Objective:** Ensure the mic button reliably starts transcription and directs it to the appropriate chat session.
* **Key Actions:**
    1.  **Direct Transcription to Current Chat:** Modify the mic button's click handler to trigger the start of a transcription session immediately.
    2.  **Chat Session Selection:** Ensure the transcription output is automatically directed to:
        * The currently active/open chat session.
        * If no chat is currently open, automatically create a new chat session and direct the transcription there.
    3.  **Transcription Flow:** Verify that interim and final transcription results are correctly displayed in the chosen chat's history.

**Phase 3: "Aubrey" Wake Word Integration (Backend & Frontend)**

* **Objective:** Implement the "Aubrey" wake word to provide a hands-free way to start transcription.
* **Key Actions:**
    1.  **Backend (Rust `speech.rs`):**
        * **Configure Wake Word Model:** Update the wake word detection logic to specifically recognize "Aubrey" as the trigger phrase. This may involve configuring an existing model or integrating a new, small wake-word-specific model.
        * **Emit Event on Detection:** When "Aubrey" is detected, the Rust backend should emit a distinct Tauri event (e.g., `aubrey-wake-word-detected`).
    2.  **Frontend (Vue Composables & Main UI):**
        * **Listen for Wake Word Event:** In a relevant composable (e.g., `useWakeWordDetection.ts`) or the main chat component, set up a listener for the `aubrey-wake-word-detected` Tauri event.
        * **Trigger Transcription:** Upon receiving the `aubrey-wake-word-detected` event, programmatically trigger the same transcription start logic as the manual mic button click (from Phase 2).
        * **Pause/Resume Logic:** Ensure that the wake word detection automatically pauses when transcription starts (either manually or via wake word) and automatically resumes once transcription finishes. This prevents re-triggering during an active conversation.