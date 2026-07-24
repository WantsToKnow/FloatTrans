@echo off
setlocal enabledelayedexpansion
:: FloatTrans — Tesseract OCR 增强引擎自动安装
:: 下载轻量英文包 (~20MB)，解压到 floattrans.exe 旁边

echo ============================================
echo   FloatTrans - Tesseract OCR 增强引擎安装
echo   这将下载约 20MB 数据，仅需一次
echo ============================================
echo.

cd /d "%~dp0"

set "TESS_DIR=rust\target\release\tesseract"
set "TESSDATA=%TESS_DIR%\tessdata"
set "ZIP=%TEMP%\tesseract-portable.zip"

:: 创建目录
if not exist "%TESSDATA%" mkdir "%TESSDATA%"

:: 下载便携版 Tesseract (UB-Mannheim)
echo [1/3] 下载 Tesseract 便携版...
powershell -Command "& {
    $url = 'https://github.com/UB-Mannheim/tesseract/releases/download/v5.5.0.20241111/tesseract-ocr-w64-portable-5.5.0.20241111.zip'
    $ProgressPreference = 'SilentlyContinue'
    try {
        Invoke-WebRequest -Uri $url -OutFile '%ZIP%' -UseBasicParsing
    } catch {
        Write-Host '[失败] 下载出错，请检查网络连接'
        Write-Host '手动下载: https://github.com/UB-Mannheim/tesseract/wiki'
        exit 1
    }
}" || goto :error

:: 解压
echo [2/3] 解压中...
powershell -Command "Expand-Archive -Path '%ZIP%' -DestinationPath '%TEMP%\tesseract_extract' -Force" || goto :error

:: 复制所需文件 (只取 eng 语言包 + exe + 必要 DLL)
echo [3/3] 复制文件...
copy /Y "%TEMP%\tesseract_extract\tesseract.exe" "%TESS_DIR%\" >nul 2>&1
copy /Y "%TEMP%\tesseract_extract\*.dll" "%TESS_DIR%\" >nul 2>&1
if exist "%TEMP%\tesseract_extract\tessdata\eng.traineddata" (
    copy /Y "%TEMP%\tesseract_extract\tessdata\eng.traineddata" "%TESSDATA%\" >nul 2>&1
)

:: 清理
del /Q "%ZIP%" >nul 2>&1
rmdir /S /Q "%TEMP%\tesseract_extract" >nul 2>&1

:: 验证
if not exist "%TESS_DIR%\tesseract.exe" goto :error
if not exist "%TESSDATA%\eng.traineddata" goto :error

echo.
echo ============================================
echo   安装完成! Tesseract OCR 已就绪
echo ============================================
echo.
pause
exit /b 0

:error
echo.
echo [失败] 安装过程中出错
echo 手动安装: 从 https://github.com/UB-Mannheim/tesseract/wiki 下载安装
echo 然后把 tesseract.exe + tessdata\eng.traineddata 放到:
echo   %TESS_DIR%
echo.
pause
exit /b 1
