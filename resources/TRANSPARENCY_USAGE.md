# 🔮 Transparency Feature Guide

## Quick Start

Click the **Command Line** button (⌘ icon) in the control panel to open transparency controls.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + T` | Toggle transparency on/off |
| `Ctrl + H` | Ghost mode (30% opacity) |
| `Ctrl + Shift + T` | Invisible mode (click-through) |
| `Esc` | Emergency restore (back to solid) |

## Features

### 🎛️ Control Panel
- **Toggle Button**: Quick on/off switch
- **Opacity Slider**: Fine-tune transparency level (0-100%)
- **Preset Buttons**: Common transparency levels
- **Emergency Restore**: Always-accessible recovery button

### 📊 Transparency Levels
- **Solid (100%)**: Normal window behavior
- **Semi-transparent (70%)**: See-through but interactive
- **Ghost Mode (30%)**: Highly transparent, still interactive
- **Invisible (0%)**: Completely see-through with click-through

### 🚨 Safety Features
- **Emergency Restore**: `Esc` key or red button always works
- **Auto-restore**: Automatically restores after 30 seconds of invisibility
- **Visual Indicators**: Color-coded status and percentage display
- **Click-through Warning**: Clear indication when clicks pass through

## Usage Tips

1. **Start with Ghost Mode** - Use `Ctrl + H` for a good balance of transparency and control
2. **Use Emergency Restore** - Press `Esc` if you lose track of the window
3. **Watch the Indicators** - Color changes show current transparency state:
   - 🟢 Green: Solid (normal)
   - 🔵 Blue: Transparent but interactive
   - 🟡 Yellow: Click-through enabled
   - 🟠 Orange: Nearly invisible
   - 🔴 Red: Error state

## Troubleshooting

**Can't see the window?**
- Press `Esc` to restore visibility
- Press `Ctrl + Shift + Esc` for emergency restore

**Clicks not working?**
- Window might be in click-through mode
- Use `Ctrl + T` to toggle back to interactive mode

**Controls not responding?**
- The transparency controls always remain interactive
- Look for the dark panel with controls - it's always clickable

## Platform Notes

- **Windows**: Full transparency and click-through support
- **macOS**: Native transparency with vibrancy effects
- **Linux**: Support varies by window manager/compositor

---

*The transparency feature automatically saves your preferences and restores them when you restart the application.* 