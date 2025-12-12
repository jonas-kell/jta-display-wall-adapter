@echo off
REM Get the directory containing this script
set "SCRIPT_DIR=%~dp0"
REM Remove trailing backslash
set "SCRIPT_DIR=%SCRIPT_DIR:~0,-1%"

REM Path to same folder
set "EXE=%SCRIPT_DIR%\jta-display-wall-adapter.exe"

REM Parameters
set "PARAMS=server --listen-to-timing-program --passthrough-address-display-program 192.168.150.10 --address-camera-program 192.168.150.10 --address-wind-server 192.168.150.10 --wait-ms-before-testing-for-shutdown 5000 --passthrough-to-display-program --address-display-client 192.168.150.150 --display-client-communication-port 5678 --play-sound-on-start --dp-pos-x=0 --dp-pos-y=0 --dp-width=360 --dp-height=120"

REM Start a new cmd window and run the exe, keep window open (/k) -> could replace with /c if cmd window should close
start "" cmd /k "%EXE% %PARAMS%"