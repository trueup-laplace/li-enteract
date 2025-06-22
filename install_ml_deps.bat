@echo off
echo Installing ML Eye Tracking Dependencies...

REM Check if Python is installed
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Python is not installed or not in PATH.
    echo Please install Python 3.8+ from https://python.org
    pause
    exit /b 1
)

REM Check Python version
for /f "tokens=2" %%i in ('python --version 2^>^&1') do set python_version=%%i
echo Found Python %python_version%

REM Upgrade pip
echo Upgrading pip...
python -m pip install --upgrade pip

REM Install core dependencies first
echo Installing core dependencies...
python -m pip install numpy>=1.24.0
python -m pip install opencv-python>=4.8.0
python -m pip install pillow>=10.0.0

REM Install ML frameworks
echo Installing ML frameworks...
python -m pip install tensorflow>=2.15.0,^<3.0.0
python -m pip install scikit-learn>=1.3.0

REM Install MediaPipe (separate due to potential conflicts)
echo Installing MediaPipe...
python -m pip install mediapipe>=0.10.8

REM Install additional dependencies
echo Installing additional dependencies...
python -m pip install dlib>=19.24.0
python -m pip install face-recognition>=1.3.0
python -m pip install numba>=0.58.0
python -m pip install scipy>=1.11.0
python -m pip install matplotlib>=3.7.0
python -m pip install psutil>=5.9.0

REM Install ONNX Runtime for optimization (optional)
echo Installing ONNX Runtime (optional)...
python -m pip install onnxruntime>=1.16.0

echo.
echo ✅ ML Eye Tracking dependencies installed successfully!
echo.
echo Testing imports...
python -c "import cv2, mediapipe, tensorflow, numpy, dlib; print('✅ All imports successful')"

if %errorlevel% equ 0 (
    echo.
    echo 🎉 Installation complete! You can now use ML Eye Tracking.
) else (
    echo.
    echo ❌ Some imports failed. Please check the error messages above.
)

pause 