@echo off
chcp 65001 >nul
echo ========================================
echo  FloatTrans build
echo ========================================
echo.

echo [1/2] dotnet build (Release)...
dotnet build FloatTrans\FloatTrans.csproj -c Release
if errorlevel 1 (
    echo.
    echo BUILD FAILED. 请确认已安装 .NET 8 SDK: https://dotnet.microsoft.com/download/dotnet/8.0
    pause
    exit /b 1
)

echo.
echo [2/2] Build OK.
echo   产物目录: FloatTrans\bin\Release\net8.0-windows10.0.19041.0\
echo.
echo 发布为单文件 exe:
echo   dotnet publish FloatTrans\FloatTrans.csproj -c Release -o publish
echo.
pause
