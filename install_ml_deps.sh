#!/bin/bash
set -e

echo "Installing ML Eye Tracking Dependencies..."

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    echo "ERROR: Python 3 is not installed or not in PATH."
    echo "Please install Python 3.8+ from your package manager or https://python.org"
    exit 1
fi

# Check Python version
python_version=$(python3 --version 2>&1 | awk '{print $2}')
echo "Found Python $python_version"

# Upgrade pip
echo "Upgrading pip..."
python3 -m pip install --upgrade pip

# Install core dependencies first
echo "Installing core dependencies..."
python3 -m pip install numpy>=1.24.0
python3 -m pip install opencv-python>=4.8.0
python3 -m pip install pillow>=10.0.0

# Install ML frameworks
echo "Installing ML frameworks..."
python3 -m pip install "tensorflow>=2.15.0,<3.0.0"
python3 -m pip install scikit-learn>=1.3.0

# Install MediaPipe (separate due to potential conflicts)
echo "Installing MediaPipe..."
python3 -m pip install mediapipe>=0.10.8

# Install additional dependencies
echo "Installing additional dependencies..."
python3 -m pip install dlib>=19.24.0
python3 -m pip install face-recognition>=1.3.0
python3 -m pip install numba>=0.58.0
python3 -m pip install scipy>=1.11.0
python3 -m pip install matplotlib>=3.7.0
python3 -m pip install psutil>=5.9.0

# Install ONNX Runtime for optimization (optional)
echo "Installing ONNX Runtime (optional)..."
python3 -m pip install onnxruntime>=1.16.0

echo
echo "✅ ML Eye Tracking dependencies installed successfully!"
echo

echo "Testing imports..."
if python3 -c "import cv2, mediapipe, tensorflow, numpy, dlib; print('✅ All imports successful')" 2>/dev/null; then
    echo
    echo "🎉 Installation complete! You can now use ML Eye Tracking."
else
    echo
    echo "❌ Some imports failed. Please check the error messages above."
    exit 1
fi 