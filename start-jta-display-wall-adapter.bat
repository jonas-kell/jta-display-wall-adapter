@echo off
REM Get the directory containing this script
set "SCRIPT_DIR=%~dp0"
REM Remove trailing backslash
set "SCRIPT_DIR=%SCRIPT_DIR:~0,-1%"

REM Path to same folder
set "EXE=%SCRIPT_DIR%\jta-display-wall-adapter.exe"

REM Parameters
set "PARAMS=server --passthrough-address-display-program 192.168.150.10 --address-camera-program 192.168.150.10 --wait-ms-before-testing-for-shutdown=1000 --passthrough-to-display-program"

REM Start a new cmd window and run the exe, keep window open (/k)
start "" cmd /k "%EXE% %PARAMS%"