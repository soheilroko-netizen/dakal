@echo off
:: Check for admin rights
net session >nul 2>&1
if %errorLevel% == 0 (
    goto :admin
) else (
    goto :elevate
)

:elevate
powershell -Command "Start-Process '%~f0' -Verb RunAs"
exit /b

:admin
:: Correct the folder path back to your .bat location
cd /d "%~dp0"

:: Run sing-box as administrator
sing-box.exe run -c config.json
pause