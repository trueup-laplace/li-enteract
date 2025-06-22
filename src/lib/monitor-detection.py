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
    """Get screen info on Windows using multiple methods"""
    monitors = []
    
    # Method 1: Try using ctypes with user32.dll (most reliable)
    try:
        import ctypes
        from ctypes import wintypes, Structure, POINTER, WINFUNCTYPE
        
        # Define structures for monitor enumeration
        class RECT(Structure):
            _fields_ = [('left', ctypes.c_long),
                      ('top', ctypes.c_long),
                      ('right', ctypes.c_long),
                      ('bottom', ctypes.c_long)]
        
        class MONITORINFO(Structure):
            _fields_ = [('cbSize', ctypes.c_ulong),
                      ('rcMonitor', RECT),
                      ('rcWork', RECT),
                      ('dwFlags', ctypes.c_ulong)]
        
        user32 = ctypes.windll.user32
        
        # Monitor enumeration callback
        MonitorEnumProc = WINFUNCTYPE(ctypes.c_bool, wintypes.HMONITOR, wintypes.HDC, POINTER(RECT), wintypes.LPARAM)
        
        def monitor_enum_callback(hmonitor, hdc, rect, data):
            try:
                monitor_info = MONITORINFO()
                monitor_info.cbSize = ctypes.sizeof(MONITORINFO)
                
                if user32.GetMonitorInfoW(hmonitor, ctypes.byref(monitor_info)):
                    rect = monitor_info.rcMonitor
                    is_primary = bool(monitor_info.dwFlags & 1)  # MONITORINFOF_PRIMARY
                    
                    monitor = Monitor(
                        x=rect.left,
                        y=rect.top,
                        width=rect.right - rect.left,
                        height=rect.bottom - rect.top,
                        is_primary=is_primary,
                        name=f'Display_{len(monitors) + 1}'
                    )
                    monitors.append(monitor)
                    print(f"DEBUG: Found monitor via ctypes: {monitor.name} at ({monitor.x}, {monitor.y}) {monitor.width}x{monitor.height} {'(PRIMARY)' if is_primary else ''}", file=sys.stderr)
            except Exception as e:
                print(f"DEBUG: Error in ctypes callback: {e}", file=sys.stderr)
            return True
        
        # Enumerate monitors
        result = user32.EnumDisplayMonitors(None, None, MonitorEnumProc(monitor_enum_callback), 0)
        print(f"DEBUG: EnumDisplayMonitors returned: {result}", file=sys.stderr)
        
        if monitors:
            print(f"DEBUG: Successfully detected {len(monitors)} monitors via ctypes", file=sys.stderr)
        
    except Exception as e:
        print(f"DEBUG: ctypes method failed: {e}", file=sys.stderr)
    
    # Method 2: Try using win32api if available
    if not monitors:
        try:
            import win32api
            import win32con
            
            print("DEBUG: Trying win32api method", file=sys.stderr)
            
            # Get virtual screen dimensions
            virtual_width = win32api.GetSystemMetrics(78)  # SM_CXVIRTUALSCREEN  
            virtual_height = win32api.GetSystemMetrics(79)  # SM_CYVIRTUALSCREEN
            virtual_left = win32api.GetSystemMetrics(76)   # SM_XVIRTUALSCREEN
            virtual_top = win32api.GetSystemMetrics(77)    # SM_YVIRTUALSCREEN
            
            print(f"DEBUG: Virtual screen via win32api: {virtual_width}x{virtual_height} at ({virtual_left}, {virtual_top})", file=sys.stderr)
            
            if virtual_width > 0 and virtual_height > 0:
                # Get primary screen dimensions
                primary_width = win32api.GetSystemMetrics(0)   # SM_CXSCREEN
                primary_height = win32api.GetSystemMetrics(1)  # SM_CYSCREEN
                
                print(f"DEBUG: Primary screen: {primary_width}x{primary_height}", file=sys.stderr)
                
                # If virtual is larger than primary, we likely have multiple monitors
                if virtual_width > primary_width or virtual_height > primary_height:
                    print("DEBUG: Multiple monitors detected via metrics comparison", file=sys.stderr)
                    
                    # Try to estimate monitor layout
                    if virtual_width > primary_width:
                        # Horizontal layout - assume two monitors side by side
                        secondary_width = virtual_width - primary_width
                        
                        # Primary monitor (usually at 0,0)
                        monitors.append(Monitor(
                            x=0, y=0,
                            width=primary_width,
                            height=primary_height,
                            is_primary=True,
                            name="Primary_Display"
                        ))
                        
                        # Secondary monitor (to the right)
                        monitors.append(Monitor(
                            x=primary_width, y=0,
                            width=secondary_width,
                            height=primary_height,  # Assume same height
                            is_primary=False,
                            name="Secondary_Display"
                        ))
                        
                    elif virtual_height > primary_height:
                        # Vertical layout - assume monitors stacked
                        secondary_height = virtual_height - primary_height
                        
                        # Primary monitor
                        monitors.append(Monitor(
                            x=0, y=0,
                            width=primary_width,
                            height=primary_height,
                            is_primary=True,
                            name="Primary_Display"
                        ))
                        
                        # Secondary monitor (below)
                        monitors.append(Monitor(
                            x=0, y=primary_height,
                            width=primary_width,
                            height=secondary_height,
                            is_primary=False,
                            name="Secondary_Display"
                        ))
                else:
                    # Single monitor
                    monitors.append(Monitor(
                        x=0, y=0,
                        width=primary_width,
                        height=primary_height,
                        is_primary=True,
                        name="Primary_Display"
                    ))
                    
                print(f"DEBUG: Created {len(monitors)} monitors via win32api estimation", file=sys.stderr)
                
        except ImportError:
            print("DEBUG: win32api not available", file=sys.stderr)
        except Exception as e:
            print(f"DEBUG: win32api method failed: {e}", file=sys.stderr)
    
    # Method 3: Try PyQt5 if available
    if not monitors:
        try:
            from PyQt5.QtWidgets import QApplication, QDesktopWidget
            import sys as qt_sys
            
            print("DEBUG: Trying PyQt5 method", file=sys.stderr)
            
            # Create QApplication if it doesn't exist
            app = QApplication.instance()
            if app is None:
                app = QApplication(qt_sys.argv)
            
            desktop = QDesktopWidget()
            screen_count = desktop.screenCount()
            
            print(f"DEBUG: PyQt5 detected {screen_count} screens", file=sys.stderr)
            
            for i in range(screen_count):
                screen_geometry = desktop.screenGeometry(i)
                is_primary = (i == desktop.primaryScreen())
                
                monitor = Monitor(
                    x=screen_geometry.x(),
                    y=screen_geometry.y(),
                    width=screen_geometry.width(),
                    height=screen_geometry.height(),
                    is_primary=is_primary,
                    name=f"Screen_{i + 1}"
                )
                monitors.append(monitor)
                print(f"DEBUG: PyQt5 screen {i}: {monitor.width}x{monitor.height} at ({monitor.x}, {monitor.y}) {'(PRIMARY)' if is_primary else ''}", file=sys.stderr)
            
        except ImportError:
            print("DEBUG: PyQt5 not available", file=sys.stderr)
        except Exception as e:
            print(f"DEBUG: PyQt5 method failed: {e}", file=sys.stderr)
    
    # Method 4: PowerShell fallback
    if not monitors:
        try:
            import subprocess
            print("DEBUG: Trying PowerShell method", file=sys.stderr)
            
            # Use PowerShell to query WMI for monitor information
            ps_command = """
            Get-WmiObject -Class Win32_DesktopMonitor | ForEach-Object {
                "$($_.Name);$($_.ScreenWidth);$($_.ScreenHeight)"
            }
            """
            
            result = subprocess.run(['powershell', '-Command', ps_command], 
                                  capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0 and result.stdout.strip():
                lines = result.stdout.strip().split('\n')
                for i, line in enumerate(lines):
                    if line.strip():
                        parts = line.split(';')
                        if len(parts) >= 3:
                            try:
                                name = parts[0] or f"Monitor_{i + 1}"
                                width = int(parts[1]) if parts[1] else 1920
                                height = int(parts[2]) if parts[2] else 1080
                                
                                monitor = Monitor(
                                    x=i * width,  # Estimate horizontal layout
                                    y=0,
                                    width=width,
                                    height=height,
                                    is_primary=(i == 0),
                                    name=name
                                )
                                monitors.append(monitor)
                                print(f"DEBUG: PowerShell monitor {i}: {width}x{height}", file=sys.stderr)
                            except ValueError:
                                continue
            
        except Exception as e:
            print(f"DEBUG: PowerShell method failed: {e}", file=sys.stderr)
    
    # If we got monitors, calculate virtual bounds
    if monitors:
        virtual_left = min(m.x for m in monitors)
        virtual_top = min(m.y for m in monitors)
        virtual_right = max(m.x + m.width for m in monitors)
        virtual_bottom = max(m.y + m.height for m in monitors)
        
        primary = next((m for m in monitors if m.is_primary), monitors[0])
        
        print(f"DEBUG: Final result: {len(monitors)} monitors, virtual bounds: {virtual_right - virtual_left}x{virtual_bottom - virtual_top}", file=sys.stderr)
        
        return ScreenInfo(
            total_width=virtual_right - virtual_left,
            total_height=virtual_bottom - virtual_top,
            virtual_left=virtual_left,
            virtual_top=virtual_top,
            monitors=monitors,
            primary_monitor=primary
        )
    
    print("DEBUG: All Windows detection methods failed, falling back", file=sys.stderr)
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
    
    # First, let's check what Windows Display Settings shows
    if platform.system().lower() == "windows":
        print("\n🔍 CHECKING WINDOWS DISPLAY SETTINGS:")
        print("-" * 40)
        try:
            import subprocess
            
            # Method 1: Check with WMIC
            print("📊 Querying WMIC for display information...")
            wmic_result = subprocess.run(['wmic', 'desktopmonitor', 'get', 'screenwidth,screenheight,name'], 
                                       capture_output=True, text=True, timeout=10)
            if wmic_result.returncode == 0:
                print("WMIC Output:")
                for line in wmic_result.stdout.split('\n'):
                    if line.strip() and 'Name' not in line:
                        print(f"  {line.strip()}")
            
            # Method 2: Check with PowerShell Get-Display
            print("\n📊 Querying PowerShell for display configuration...")
            ps_display_cmd = """
            try {
                Add-Type -AssemblyName System.Windows.Forms
                $screens = [System.Windows.Forms.Screen]::AllScreens
                foreach ($screen in $screens) {
                    Write-Output "Screen: $($screen.DeviceName) - $($screen.Bounds.Width)x$($screen.Bounds.Height) at ($($screen.Bounds.X),$($screen.Bounds.Y)) Primary:$($screen.Primary)"
                }
            } catch {
                Write-Output "Error: $_"
            }
            """
            
            ps_result = subprocess.run(['powershell', '-Command', ps_display_cmd], 
                                     capture_output=True, text=True, timeout=10)
            if ps_result.returncode == 0:
                print("PowerShell Output:")
                for line in ps_result.stdout.split('\n'):
                    if line.strip():
                        print(f"  {line.strip()}")
            
            # Method 3: Registry check
            print("\n📊 Checking Windows Registry for display settings...")
            reg_cmd = 'reg query "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers\\Configuration" /s'
            try:
                reg_result = subprocess.run(reg_cmd, shell=True, capture_output=True, text=True, timeout=10)
                if reg_result.returncode == 0:
                    # Look for resolution patterns
                    import re
                    resolution_pattern = r'(\d{3,4})x(\d{3,4})'
                    resolutions = re.findall(resolution_pattern, reg_result.stdout)
                    unique_resolutions = list(set(resolutions))
                    print(f"Registry resolutions found: {unique_resolutions}")
                else:
                    print("Registry query failed")
            except Exception as e:
                print(f"Registry check failed: {e}")
                
        except Exception as e:
            print(f"Windows detection debug failed: {e}")
        
        print("-" * 40)
    
    try:
        screen_info = get_screen_info()
        print_screen_info(screen_info)
        
        # Show what your eye tracking would use
        width, height = get_auto_screen_dimensions()
        print(f"\n✅ Eye tracking should use: {width} x {height}")
        
        # Test different scenarios
        print(f"\n🎯 FOR YOUR EYE TRACKING SCRIPT:")
        print(f"python eye-tracking-ml.py --screen-width {width} --screen-height {height}")
        
        # Additional diagnostic information
        print(f"\n🔧 DIAGNOSTIC INFORMATION:")
        print("-" * 30)
        
        if platform.system().lower() == "windows":
            try:
                import ctypes
                user32 = ctypes.windll.user32
                
                # Get various system metrics
                metrics = {
                    "Primary Screen Width": user32.GetSystemMetrics(0),
                    "Primary Screen Height": user32.GetSystemMetrics(1),
                    "Virtual Screen Width": user32.GetSystemMetrics(78),
                    "Virtual Screen Height": user32.GetSystemMetrics(79),
                    "Virtual Screen Left": user32.GetSystemMetrics(76),
                    "Virtual Screen Top": user32.GetSystemMetrics(77),
                    "Number of Monitors": user32.GetSystemMetrics(80),
                }
                
                for key, value in metrics.items():
                    print(f"{key}: {value}")
                    
                # Check if we can detect multiple monitors
                if metrics["Number of Monitors"] > 1:
                    print(f"\n✅ Windows reports {metrics['Number of Monitors']} monitors!")
                    print(f"   Virtual desktop: {metrics['Virtual Screen Width']}x{metrics['Virtual Screen Height']}")
                    print(f"   Primary screen: {metrics['Primary Screen Width']}x{metrics['Primary Screen Height']}")
                else:
                    print("\n⚠️  Windows reports only 1 monitor")
                    
            except Exception as e:
                print(f"System metrics check failed: {e}")
        
        # Try alternative detection methods
        print(f"\n🧪 TESTING ALTERNATIVE DETECTION METHODS:")
        print("-" * 40)
        
        # Test tkinter
        try:
            import tkinter as tk
            root = tk.Tk()
            tk_width = root.winfo_screenwidth()
            tk_height = root.winfo_screenheight()
            
            # Try virtual root
            try:
                vroot_width = root.winfo_vrootwidth()
                vroot_height = root.winfo_vrootheight()
                print(f"Tkinter virtual root: {vroot_width}x{vroot_height}")
            except:
                print(f"Tkinter virtual root: Not available")
            
            print(f"Tkinter screen: {tk_width}x{tk_height}")
            root.destroy()
        except Exception as e:
            print(f"Tkinter test failed: {e}")
        
        # Test PyQt5 if available
        try:
            from PyQt5.QtWidgets import QApplication, QDesktopWidget
            import sys as qt_sys
            
            app = QApplication.instance() or QApplication(qt_sys.argv)
            desktop = QDesktopWidget()
            
            print(f"PyQt5 screen count: {desktop.screenCount()}")
            for i in range(desktop.screenCount()):
                geom = desktop.screenGeometry(i)
                print(f"  Screen {i}: {geom.width()}x{geom.height()} at ({geom.x()}, {geom.y()})")
                
        except ImportError:
            print("PyQt5: Not available")
        except Exception as e:
            print(f"PyQt5 test failed: {e}")
        
    except Exception as e:
        print(f"❌ Error detecting screens: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()