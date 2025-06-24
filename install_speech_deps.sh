#!/bin/bash

echo "Installing Python dependencies for Enteract speech functionality..."
echo

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    if ! command -v python &> /dev/null; then
        echo "Python is not installed. Please install Python 3.8+ first."
        exit 1
    else
        PYTHON_CMD="python"
    fi
else
    PYTHON_CMD="python3"
fi

echo "Python found: $($PYTHON_CMD --version)"
echo "Installing dependencies..."
echo

# Check if pip is available
if ! command -v pip3 &> /dev/null; then
    if ! command -v pip &> /dev/null; then
        echo "pip is not installed. Please install pip first."
        exit 1
    else
        PIP_CMD="pip"
    fi
else
    PIP_CMD="pip3"
fi

echo "Installing audio processing dependencies..."
$PIP_CMD install pyaudio numpy

echo "Installing speech recognition dependencies..."
$PIP_CMD install openai-whisper

echo "Installing additional audio processing..."
$PIP_CMD install scipy

echo
echo "Speech dependencies installation complete!"
echo
echo "To start the application with speech support:"
echo "1. Run 'npm run tauri dev' or build the app"
echo "2. Use the speech controls in the app interface"
echo "3. Say 'Aubrey' to activate speech recognition"
echo

# Note for Linux users about PortAudio
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Note for Linux users:"
    echo "If pyaudio installation fails, you may need to install PortAudio development headers:"
    echo "  Ubuntu/Debian: sudo apt-get install portaudio19-dev"
    echo "  CentOS/RHEL: sudo yum install portaudio-devel"
    echo "  Arch: sudo pacman -S portaudio"
fi 