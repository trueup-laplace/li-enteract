# Embedded Agentic Assistant

## 🤖 Overview

The **Embedded Agentic Assistant** is a next-generation, hyper-personalized desktop application that seamlessly blends web, native, and 3D interfaces to create an active, intelligent layer over your computing environment. Unlike passive assistants, this embedded system **observes**, **predicts**, and **augments** your workflow in real-time.

### ✨ Key Features
- **🎨 Decorative & Interactive UI:** Frameless window with animated 3D visuals and glassmorphism effects
- **👀 Eye-driven Interaction:** Move windows, adjust elements, or manipulate views just by looking
- **🎹 OS-level Actuation:** Type, click, and control applications programmatically based on context and intent
- **🖥️ Persistent but Non-intrusive:** Runs quietly, surfaces when needed, integrates deeply into your desktop environment
- **🧠 AI-Powered:** Contextually aware assistant that learns and adapts to your workflow patterns

*It's not just an assistant. It's an **extension of your will**—embedded, intelligent, beautiful.*

## 🛠️ Tech Stack
<img width="1031" height="741" alt="image" src="https://github.com/user-attachments/assets/ebcb93f3-53b5-420c-91f6-c586d53c7ea3" />

- **Frontend:** Vue 3 + TypeScript + Vite
- **Backend:** Rust + Tauri
- **3D Graphics:** Three.js
- **Styling:** TailwindCSS + Custom themes + DaisyUI
- **Eye Tracking:** OpenCV.js + MediaDevices API
- **Animations:** GSAP + CSS transitions
- **State Management:** Pinia

## 🚀 Quick Start

### Prerequisites
- **Node.js** 18+
- **Rust** (latest stable)
- **Visual Studio Build Tools** (Windows) or **Xcode CLI Tools** (macOS)

### Installation
```bash
# Clone the repository
git clone <repository-url>
cd embedded-agentic-assistant

# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run tauri build
```

### Setup from Scratch
```bash
# Create new project
npx create-tauri-app@latest embedded-agentic-assistant
# Choose: TypeScript → Vue → TypeScript

cd embedded-agentic-assistant

# Install additional dependencies
npm install three @types/three tailwindcss @headlessui/vue @heroicons/vue gsap opencv-js lodash @types/lodash uuid date-fns

# Initialize Tailwind
npx tailwindcss init -p

# Start development
npm run tauri dev
```

## 📁 Project Structure

```
embedded-agentic-assistant/
├── src/                     # Vue 3 + TypeScript frontend
│   ├── components/          # UI components
│   │   ├── core/           # Base UI components
│   │   ├── three/          # 3D scene components
│   │   ├── interaction/    # Eye tracking & gestures
│   │   └── overlay/        # Desktop overlays
│   ├── composables/        # Vue composables
│   │   ├── useEyeTracking.ts
│   │   ├── useThreeScene.ts
│   │   └── useWindowManager.ts
│   ├── stores/             # Pinia state stores
│   ├── utils/              # Utility functions
│   └── types/              # TypeScript definitions
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri commands
│   │   │   ├── window_manager.rs
│   │   │   ├── os_automation.rs
│   │   │   └── system_info.rs
│   │   └── main.rs
│   └── tauri.conf.json     # Tauri configuration
├── vite.config.ts          # Vite configuration
├── tailwind.config.js      # Tailwind CSS config
└── package.json
```

## ⚡ Development Scripts

```bash
npm run tauri dev      # Start development server
npm run tauri build    # Build for production
npm run dev           # Frontend only (for UI development)
npm run build         # Build frontend only
npm run lint          # Lint code
npm run type-check    # TypeScript validation
```

## 🎨 Configuration

### Window Settings
The application runs as a frameless, transparent window with:
- Custom title bar with drag regions
- Always-on-top capability
- System tray integration
- Smooth animations and transitions

### Permissions
Required for full functionality:
- Camera access (eye tracking)
- Screen recording (context awareness)
- Accessibility permissions (OS automation)
- File system access

## 🔧 Development Features

### Hot Reload
- Instant frontend updates with Vite HMR
- Automatic Rust recompilation
- Live reload for configuration changes

### Code Splitting
- Optimized Three.js chunks
- Separate vendor bundles
- Lazy-loaded components

### TypeScript
- Full type safety across frontend and Tauri APIs
- Auto-completion for all libraries
- Build-time error checking

## 🎯 Roadmap

### Phase 1: Foundation ✅
- Basic Tauri + Vue setup
- Frameless window with custom decorations
- 3D scene integration

### Phase 2: Interaction (In Progress)
- Eye tracking implementation
- Gesture recognition
- Window management automation

### Phase 3: Intelligence (Planned)
- Local AI integration
- Context awareness
- Predictive workflows

### Phase 4: Advanced Features (Future)
- Voice commands
- Multi-monitor support
- Plugin system

## 📚 Documentation

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Vue 3 Composition API](https://vuejs.org/guide/extras/composition-api-faq.html)
- [Three.js Documentation](https://threejs.org/docs/)
- [TailwindCSS Documentation](https://tailwindcss.com/docs)

---

*Building the future of human-computer interaction, one pixel at a time.* ✨
