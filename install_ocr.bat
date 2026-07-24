@echo off
:: FloatTrans — 安装英文 OCR 语言包
:: 需要管理员权限运行

echo ============================================
echo   FloatTrans OCR 语言包安装脚本
echo   正在安装英文 OCR 识别引擎...
echo ============================================
echo.

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 请右键此文件 → "以管理员身份运行"
    echo.
    pause
    exit /b 1
)

echo 正在查找英文 OCR 语言包...
for /f "tokens=*" %%i in ('powershell -Command "(Get-WindowsCapability -Online | Where-Object { $_.Name -Like 'Language.OCR*en-US*' }).Name"') do set CAP=%%i

if "%CAP%"=="" (
    echo [失败] 未找到英文 OCR 语言包, 请手动安装:
    echo   设置 → 时间和语言 → 语言 → 添加语言
    echo   → English ^(United States^) → 勾选 OCR
    echo.
    pause
    exit /b 1
)

echo 找到: %CAP%
echo 正在安装...
powershell -Command "Add-WindowsCapability -Online -Name '%CAP%'"

if %errorlevel% equ 0 (
    echo.
    echo ============================================
    echo   安装完成! 现在可以运行 floattrans.exe 了
    echo ============================================
) else (
    echo.
    echo [失败] 安装出错, 请尝试手动安装:
    echo   设置 → 时间和语言 → 语言 → 添加语言
    echo   → English ^(United States^) → 勾选 OCR
)

echo.
pause
