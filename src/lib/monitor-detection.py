#!/usr/bin/env python3
"""
Multi-Monitor Screen Detection Utility
Automatically detects all connected monitors and calculates total screen dimensions
Cross-platform support for Windows, macOS, and Linux
"""

import sys
import platform
from typing import List, Tuple, Dict, Optional
from dataclasses import dataclass

@dataclass
class Monitor:
    """Represents a single monitor"""
    x: int
    y: int
    width: int
    height: int
    is_primary: bool = False
    name: str = ""
    scale_factor: float = 1.0

@dataclass
class ScreenInfo:
    """Complete screen configuration info"""
    total_width: int
    total_height: int
    virtual_left: int
    virtual_top: int
    monitors: List[Monitor]
    primary_monitor: Optional[Monitor] = None

def get_screen_info() -> ScreenInfo:
    """
    Get comprehensive screen information for all connected monitors
    Returns total virtual screen dimensions and individual monitor details
    """
    system = platform.system().lower()
    
    if system == "windows":
        return _get_windows_screen_info()
    elif system == "darwin":  # macOS
        return _get_macos_screen_info()
    elif system == "linux":
        return _get_linux_screen_info()
    else:
        # Fallback to single monitor detection
        return _get_fallback_screen_info()

def _get_windows_screen_info() -> ScreenInfo:
    """Get screen info on Windows using win32api"""
    try:
        import win32api
        import win32con
        import win32gui
        
        monitors = []
        
        def monitor_enum_proc(hmonitor, hdc, rect, data):
            """Callback for EnumDisplayMonitors"""
            monitor_info = win32api.GetMonitorInfo(hmonitor)
            
            # Get monitor rectangle
            monitor_rect = monitor_info['Monitor']
            work_rect = monitor_info['Work']
            
            # Check if this is the primary monitor
            is_primary = monitor_info['Flags'] & win32con.MONITORINFOF_PRIMARY
            
            monitor = Monitor(
                x=monitor_rect[0],
                y=monitor_rect[1], 
                width=monitor_rect[2] - monitor_rect[0],
                height=monitor_rect[3] - monitor_rect[1],
                is_primary=bool(is_primary),
                name=f"Monitor {len(monitors) + 1}"
            )
            
            monitors.append(monitor)
            return True
        
        # Enumerate all monitors
        win32gui.EnumDisplayMonitors(None, None, monitor_enum_proc, None)
        
        # Calculate virtual screen bounds
        virtual_left = min(m.x for m in monitors)
        virtual_top = min(m.y for m in monitors)
        virtual_right = max(m.x + m.width for m in monitors)
        virtual_bottom = max(m.y + m.height for m in monitors)
        
        # Find primary monitor
        primary = next((m for m in monitors if m.is_primary), monitors[0] if monitors else None)
        
        return ScreenInfo(
            total_width=virtual_right - virtual_left,
            total_height=virtual_bottom - virtual_top,
            virtual_left=virtual_left,
            virtual_top=virtual_top,
            monitors=monitors,
            primary_monitor=primary
        )
        
    except ImportError:
        print("WARNING: win32api not available, falling back to basic detection", file=sys.stderr)
        return _get_fallback_screen_info()
    except Exception as e:
        print(f"WARNING: Windows screen detection failed: {e}", file=sys.stderr)
        return _get_fallback_screen_info()

def _get_macos_screen_info() -> ScreenInfo:
    """Get screen info on macOS using Quartz"""
    try:
        from Quartz import CGGetActiveDisplayList, CGDisplayBounds, CGMainDisplayID
        
        # Get all active displays
        max_displays = 10
        (err, active_displays, num_displays) = CGGetActiveDisplayList(max_displays, None, None)
        
        if err != 0:
            raise Exception(f"CGGetActiveDisplayList failed with error {err}")
        
        monitors = []
        main_display_id = CGMainDisplayID()
        
        for display_id in active_displays[:num_displays]:
            bounds = CGDisplayBounds(display_id)
            
            is_primary = (display_id == main_display_id)
            
            monitor = Monitor(
                x=int(bounds.origin.x),
                y=int(bounds.origin.y),
                width=int(bounds.size.width),
                height=int(bounds.size.height),
                is_primary=is_primary,
                name=f"Display {display_id}"
            )
            
            monitors.append(monitor)
        
        # Calculate virtual screen bounds
        virtual_left = min(m.x for m in monitors)
        virtual_top = min(m.y for m in monitors)
        virtual_right = max(m.x + m.width for m in monitors)
        virtual_bottom = max(m.y + m.height for m in monitors)
        
        # Find primary monitor
        primary = next((m for m in monitors if m.is_primary), monitors[0] if monitors else None)
        
        return ScreenInfo(
            total_width=virtual_right - virtual_left,
            total_height=virtual_bottom - virtual_top,
            virtual_left=virtual_left,
            virtual_top=virtual_top,
            monitors=monitors,
            primary_monitor=primary
        )
        
    except ImportError:
        print("WARNING: Quartz not available, falling back to basic detection", file=sys.stderr)
        return _get_fallback_screen_info()
    except Exception as e:
        print(f"WARNING: macOS screen detection failed: {e}", file=sys.stderr)
        return _get_fallback_screen_info()

def _get_linux_screen_info() -> ScreenInfo:
    """Get screen info on Linux using X11"""
    try:
        # Try multiple methods for Linux
        
        # Method 1: Try using tkinter (most compatible)
        try:
            import tkinter as tk
            root = tk.Tk()
            
            # Get screen dimensions
            screen_width = root.winfo_screenwidth()
            screen_height = root.winfo_screenheight()
            
            # Try to get more detailed info if available
            monitors = [Monitor(
                x=0, y=0, 
                width=screen_width, 
                height=screen_height, 
                is_primary=True,
                name="Primary Display"
            )]
            
            root.destroy()
            
            return ScreenInfo(
                total_width=screen_width,
                total_height=screen_height,
                virtual_left=0,
                virtual_top=0,
                monitors=monitors,
                primary_monitor=monitors[0]
            )
            
        except ImportError:
            pass
        
        # Method 2: Try using Xlib
        try:
            from Xlib import display
            from Xlib.ext import randr
            
            d = display.Display()
            screen = d.screen()
            
            # Get RandR extension info
            res = randr.get_screen_resources(screen.root)
            monitors = []
            
            for output in res.outputs:
                output_info = randr.get_output_info(screen.root, output, res.config_timestamp)
                if output_info.crtc:
                    crtc_info = randr.get_crtc_info(screen.root, output_info.crtc, res.config_timestamp)
                    
                    monitor = Monitor(
                        x=crtc_info.x,
                        y=crtc_info.y,
                        width=crtc_info.width,
                        height=crtc_info.height,
                        is_primary=(len(monitors) == 0),  # First one is primary
                        name=output_info.name or f"Output {len(monitors) + 1}"
                    )
                    monitors.append(monitor)
            
            if monitors:
                virtual_left = min(m.x for m in monitors)
                virtual_top = min(m.y for m in monitors)
                virtual_right = max(m.x + m.width for m in monitors)
                virtual_bottom = max(m.y + m.height for m in monitors)
                
                return ScreenInfo(
                    total_width=virtual_right - virtual_left,
                    total_height=virtual_bottom - virtual_top,
                    virtual_left=virtual_left,
                    virtual_top=virtual_top,
                    monitors=monitors,
                    primary_monitor=monitors[0]
                )
        
        except ImportError:
            pass
        
        # Method 3: Try subprocess with xrandr
        try:
            import subprocess
            result = subprocess.run(['xrandr'], capture_output=True, text=True)
            
            if result.returncode == 0:
                monitors = []
                for line in result.stdout.split('\n'):
                    if ' connected' in line and ('primary' in line or len(monitors) == 0):
                        # Parse resolution from xrandr output
                        parts = line.split()
                        for part in parts:
                            if 'x' in part and '+' in part:
                                # Format like "1920x1080+0+0"
                                res_part, pos_part = part.split('+', 1)
                                width, height = map(int, res_part.split('x'))
                                x_pos = int(pos_part.split('+')[0])
                                y_pos = int(pos_part.split('+')[1]) if '+' in pos_part else 0
                                
                                monitor = Monitor(
                                    x=x_pos, y=y_pos,
                                    width=width, height=height,
                                    is_primary='primary' in line,
                                    name=parts[0]
                                )
                                monitors.append(monitor)
                                break
                
                if monitors:
                    virtual_left = min(m.x for m in monitors)
                    virtual_top = min(m.y for m in monitors)
                    virtual_right = max(m.x + m.width for m in monitors)
                    virtual_bottom = max(m.y + m.height for m in monitors)
                    
                    return ScreenInfo(
                        total_width=virtual_right - virtual_left,
                        total_height=virtual_bottom - virtual_top,
                        virtual_left=virtual_left,
                        virtual_top=virtual_top,
                        monitors=monitors,
                        primary_monitor=next((m for m in monitors if m.is_primary), monitors[0])
                    )
        
        except Exception:
            pass
        
        # Fallback for Linux
        return _get_fallback_screen_info()
        
    except Exception as e:
        print(f"WARNING: Linux screen detection failed: {e}", file=sys.stderr)
        return _get_fallback_screen_info()

def _get_fallback_screen_info() -> ScreenInfo:
    """Fallback method using basic detection"""
    try:
        # Try tkinter first
        import tkinter as tk
        root = tk.Tk()
        width = root.winfo_screenwidth()
        height = root.winfo_screenheight()
        root.destroy()
        
        monitor = Monitor(
            x=0, y=0, width=width, height=height, 
            is_primary=True, name="Primary Display"
        )
        
        return ScreenInfo(
            total_width=width,
            total_height=height,
            virtual_left=0,
            virtual_top=0,
            monitors=[monitor],
            primary_monitor=monitor
        )
        
    except Exception:
        # Ultimate fallback
        print("WARNING: Using default screen dimensions", file=sys.stderr)
        monitor = Monitor(
            x=0, y=0, width=1920, height=1080,
            is_primary=True, name="Default Display"
        )
        
        return ScreenInfo(
            total_width=1920,
            total_height=1080,
            virtual_left=0,
            virtual_top=0,
            monitors=[monitor],
            primary_monitor=monitor
        )

def print_screen_info(screen_info: ScreenInfo):
    """Print detailed screen information"""
    print("\n" + "="*50)
    print("🖥️  SCREEN CONFIGURATION")
    print("="*50)
    
    print(f"📏 Total Virtual Screen: {screen_info.total_width} x {screen_info.total_height}")
    print(f"📍 Virtual Bounds: ({screen_info.virtual_left}, {screen_info.virtual_top})")
    print(f"🖱️  Number of Monitors: {len(screen_info.monitors)}")
    
    print("\n📺 INDIVIDUAL MONITORS:")
    print("-" * 30)
    
    for i, monitor in enumerate(screen_info.monitors, 1):
        primary_indicator = " (PRIMARY)" if monitor.is_primary else ""
        print(f"Monitor {i}: {monitor.name}{primary_indicator}")
        print(f"  Position: ({monitor.x}, {monitor.y})")
        print(f"  Size: {monitor.width} x {monitor.height}")
        print(f"  Scale: {monitor.scale_factor}x")
        print()
    
    # Show layout visualization
    print("🗺️  LAYOUT VISUALIZATION:")
    print("-" * 30)
    
    # Simple ASCII representation
    if len(screen_info.monitors) > 1:
        print("Multiple monitor setup detected!")
        for monitor in screen_info.monitors:
            position = "Left" if monitor.x < 0 else "Right" if monitor.x > 0 else "Center"
            print(f"  {monitor.name}: {position} ({monitor.width}x{monitor.height})")
    else:
        print("Single monitor setup")
    
    print("="*50)

def get_auto_screen_dimensions() -> Tuple[int, int]:
    """
    Convenience function that returns total screen width and height
    This is what you'd typically use in your eye tracking script
    """
    screen_info = get_screen_info()
    return screen_info.total_width, screen_info.total_height

def main():
    """Test the screen detection functionality"""
    print("🔍 Detecting screen configuration...")
    
    try:
        screen_info = get_screen_info()
        print_screen_info(screen_info)
        
        # Show what your eye tracking would use
        width, height = get_auto_screen_dimensions()
        print(f"\n✅ Eye tracking should use: {width} x {height}")
        
        # Test different scenarios
        print(f"\n🎯 FOR YOUR EYE TRACKING SCRIPT:")
        print(f"python eye-tracking-ml.py --screen-width {width} --screen-height {height}")
        
    except Exception as e:
        print(f"❌ Error detecting screens: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()