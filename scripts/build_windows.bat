@echo off
REM Windows 构建脚本 - 用于GitHub Actions和本地构建
REM 开发心理：为Windows用户提供简单的一键构建体验

echo 🚀 Pokemon GO Windows 构建器
echo =====================================

REM 设置构建参数
set BUILD_MODE=Release
set BUILD_NATIVE=true
set RUN_TESTS=false

REM 解析命令行参数
:parse_args
if "%1"=="--debug" (
    set BUILD_MODE=Debug
    shift
    goto parse_args
)
if "%1"=="--test" (
    set RUN_TESTS=true
    shift
    goto parse_args
)
if "%1"=="--no-native" (
    set BUILD_NATIVE=false
    shift
    goto parse_args
)
if "%1"=="" goto start_build
shift
goto parse_args

:start_build
echo 构建模式: %BUILD_MODE%
echo 构建C++模块: %BUILD_NATIVE%
echo 运行测试: %RUN_TESTS%

REM 检查依赖
echo.
echo 📦 检查构建依赖...

where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ 错误: 需要安装 Rust/Cargo
    echo    请访问 https://rustup.rs/ 安装
    exit /b 1
)
echo ✓ Cargo 已安装

if "%BUILD_NATIVE%"=="true" (
    where cmake >nul 2>&1
    if %ERRORLEVEL% NEQ 0 (
        echo ❌ 错误: 需要安装 CMake
        echo    请访问 https://cmake.org/download/ 安装
        exit /b 1
    )
    echo ✓ CMake 已安装
    
    REM 检查Visual Studio构建工具
    if not exist "%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" (
        echo ❌ 错误: 需要安装 Visual Studio Build Tools
        echo    请安装 Visual Studio 2019/2022 或 Build Tools
        exit /b 1
    )
    echo ✓ Visual Studio Build Tools 已安装
)

REM 构建C++模块
if "%BUILD_NATIVE%"=="true" (
    echo.
    echo 🔧 构建C++高性能模块...
    
    if not exist build mkdir build
    cd build
    
    if "%BUILD_MODE%"=="Release" (
        cmake .. -DCMAKE_BUILD_TYPE=Release -A x64
    ) else (
        cmake .. -DCMAKE_BUILD_TYPE=Debug -A x64
    )
    
    if %ERRORLEVEL% NEQ 0 (
        echo ❌ CMake 配置失败
        cd ..
        exit /b 1
    )
    
    cmake --build . --config %BUILD_MODE% --parallel
    
    if %ERRORLEVEL% NEQ 0 (
        echo ❌ C++模块构建失败
        cd ..
        exit /b 1
    )
    
    cd ..
    echo ✓ C++模块构建完成
)

REM 构建Rust项目
echo.
echo 🦀 构建Rust游戏引擎...

if "%BUILD_MODE%"=="Release" (
    cargo build --release --no-default-features
) else (
    cargo build --no-default-features
)

if %ERRORLEVEL% NEQ 0 (
    echo ❌ Rust项目构建失败
    exit /b 1
)
echo ✓ Rust项目构建完成

REM 运行测试
if "%RUN_TESTS%"=="true" (
    echo.
    echo 🧪 运行测试套件...
    
    cargo test --no-default-features
    
    if %ERRORLEVEL% NEQ 0 (
        echo ❌ 测试失败
        exit /b 1
    )
    echo ✓ 测试完成
)

REM 生成构建报告
echo.
echo 📊 构建报告
echo ===================
echo 构建模式: %BUILD_MODE%
echo C++模块: %BUILD_NATIVE%
echo 测试状态: %RUN_TESTS%

if "%BUILD_MODE%"=="Release" (
    set BINARY_PATH=target\release\pokemongo.exe
) else (
    set BINARY_PATH=target\debug\pokemongo.exe
)

if exist "%BINARY_PATH%" (
    for %%I in ("%BINARY_PATH%") do echo 二进制大小: %%~zI bytes
    echo 可执行文件: %BINARY_PATH%
) else (
    echo ⚠️  未找到可执行文件
)

echo 构建时间: %date% %time%

echo.
echo 🎉 Windows构建完成！
echo 运行游戏: %BINARY_PATH%
echo 或者运行: cargo run --bin pokemongo --no-default-features

pause