@echo off
setlocal EnableExtensions
cd /d "%~dp0.."
if errorlevel 1 exit /b 1

where cargo >nul 2>nul
if errorlevel 1 (
  echo ERROR: cargo.exe not found. Install Rust stable MSVC, then reopen the terminal.
  exit /b 1
)

echo [1/3] Checking Rust core tests...
cargo test -p limers_core
if errorlevel 1 exit /b 1

echo [2/3] Building LimeRS.exe, 64-bit Windows recommended...
cargo build --release -p limers-portable
if errorlevel 1 (
  echo ERROR: Rust compilation failed. Read the first error above.
  exit /b 1
)

echo [3/3] Packaging portable files...
if not exist "portable\data" mkdir "portable\data"
copy /Y "target\release\LimeRS.exe" "portable\LimeRS.exe" >nul
if errorlevel 1 exit /b 1

echo.
echo SUCCESS: portable\LimeRS.exe
 echo Run it by double-clicking the EXE; no regsvr32 or TSF registration required.
echo Toggle on/off with Ctrl+Alt+F9 or the notification area icon.
endlocal
