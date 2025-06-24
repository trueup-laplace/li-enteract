@echo off
echo Installing Python dependencies for Enteract speech functionality...
echo.

echo Checking Python installation...
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Python is not installed or not in PATH. Please install Python 3.8+ first.
    pause
    exit /b 1
)

echo Python found. Installing dependencies...
echo.

echo Installing audio processing dependencies...
pip install pyaudio numpy

echo Installing speech recognition dependencies...
pip install openai-whisper

echo Installing additional audio processing...
pip install scipy

echo.
echo Speech dependencies installation complete!
echo.
echo To start the application with speech support:
echo 1. Run 'npm run tauri dev' or build the app
echo 2. Use the speech controls in the app interface
echo 3. Say "Aubrey" to activate speech recognition
echo.

pause 