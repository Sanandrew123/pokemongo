@echo off
REM Windows 安装包创建脚本
REM 开发心理：为Windows用户提供专业的安装体验

echo 📦 创建 Pokemon GO Windows 安装包
echo ======================================

REM 检查是否已构建
if not exist "target\release\pokemongo.exe" (
    echo ❌ 错误: 请先运行构建脚本
    echo 运行: scripts\build_windows.bat --release
    exit /b 1
)

REM 创建安装目录结构
echo 📁 创建安装目录结构...
if exist "dist" rmdir /s /q dist
mkdir dist
mkdir dist\bin
mkdir dist\assets
mkdir dist\docs

REM 复制文件
echo 📋 复制游戏文件...
copy "target\release\pokemongo.exe" "dist\bin\"
copy "target\release\*.dll" "dist\bin\" 2>nul || echo "No DLLs to copy"

REM 复制资源文件
xcopy "assets\*" "dist\assets\" /E /I /Y

REM 复制文档
copy "README.md" "dist\docs\"
copy "GAME_MODES.md" "dist\docs\"
copy "CREATURE_SYSTEM.md" "dist\docs\"

REM 创建启动器
echo 🚀 创建游戏启动器...
(
echo @echo off
echo title Pokemon GO - 高性能宝可梦游戏
echo cd /d "%%~dp0bin"
echo echo.
echo echo 🎮 ==========================================
echo echo 🎮     Pokemon GO - 高性能宝可梦游戏
echo echo 🎮 ==========================================
echo echo.
echo echo 正在启动游戏...
echo echo.
echo start "" pokemongo.exe
echo echo.
echo echo 游戏已启动！如有问题请查看 docs 文件夹中的文档。
echo echo.
echo pause
) > "dist\Pokemon GO.bat"

REM 创建卸载脚本
echo 🗑️ 创建卸载脚本...
(
echo @echo off
echo title Pokemon GO - 卸载程序
echo echo.
echo echo 🗑️ Pokemon GO 卸载程序
echo echo ========================
echo echo.
echo set /p confirm="确定要卸载 Pokemon GO 吗？ (y/N): "
echo if /i "%%confirm%%"=="y" (
echo     echo.
echo     echo 正在删除游戏文件...
echo     cd /d "%%~dp0.."
echo     rmdir /s /q "%%cd%%"
echo     echo 卸载完成！
echo ) else (
echo     echo 取消卸载。
echo )
echo pause
) > "dist\uninstall.bat"

REM 创建系统信息脚本
echo 💻 创建系统信息脚本...
(
echo @echo off
echo title Pokemon GO - 系统信息
echo echo.
echo echo 💻 Pokemon GO - 系统信息检查
echo echo ================================
echo echo.
echo echo 操作系统:
echo ver
echo echo.
echo echo 处理器信息:
echo wmic cpu get name /value ^| find "Name="
echo echo.
echo echo 内存信息:
echo wmic memorychip get capacity /value ^| find "Capacity="
echo echo.
echo echo 显卡信息:
echo wmic path win32_VideoController get name /value ^| find "Name="
echo echo.
echo echo DirectX版本:
echo dxdiag /t dxdiag_output.txt
echo if exist dxdiag_output.txt (
echo     findstr "DirectX Version" dxdiag_output.txt
echo     del dxdiag_output.txt
echo )
echo echo.
echo echo ================================
echo echo 如果遇到性能问题，请将此信息提供给技术支持。
echo echo.
echo pause
) > "dist\system_info.bat"

REM 创建README文件
echo 📝 创建Windows安装说明...
(
echo # 🎮 Pokemon GO - 高性能宝可梦游戏
echo.
echo ## 🚀 快速开始
echo.
echo 1. **启动游戏**: 双击 `Pokemon GO.bat`
echo 2. **查看文档**: 打开 `docs` 文件夹
echo 3. **系统检查**: 运行 `system_info.bat`
echo 4. **卸载游戏**: 运行 `uninstall.bat`
echo.
echo ## 💻 系统要求
echo.
echo - **操作系统**: Windows 10/11 (64位)
echo - **内存**: 4GB RAM (推荐 8GB+)
echo - **显卡**: DirectX 12 兼容显卡
echo - **存储**: 1GB 可用空间
echo - **网络**: 可选 (多人模式需要)
echo.
echo ## 🎮 游戏控制
echo.
echo - **Enter**: 进入游戏
echo - **ESC**: 暂停/返回
echo - **B**: 进入战斗模式
echo - **Alt+F4**: 退出游戏
echo.
echo ## 🔧 故障排除
echo.
echo ### 游戏无法启动
echo - 确保系统已安装最新的 Visual C++ Redistributable
echo - 运行 `system_info.bat` 检查系统配置
echo - 查看 Windows 事件查看器中的错误日志
echo.
echo ### 性能问题
echo - 更新显卡驱动程序
echo - 关闭其他占用资源的程序
echo - 在游戏设置中降低图形质量
echo.
echo ### 音频问题
echo - 检查音频设备是否正常工作
echo - 更新音频驱动程序
echo - 在游戏设置中调整音频选项
echo.
echo ## 📞 技术支持
echo.
echo 如遇问题，请访问项目 GitHub 页面或联系开发团队。
echo.
echo ---
echo 版本: Windows Release
echo 构建时间: %date% %time%
) > "dist\README_Windows.txt"

echo.
echo ✅ Windows安装包创建完成！
echo.
echo 📦 安装包位置: dist\
echo 📁 包含文件:
echo    - Pokemon GO.bat (启动器)
echo    - bin\ (游戏文件)
echo    - assets\ (游戏资源)
echo    - docs\ (文档)
echo    - system_info.bat (系统信息)
echo    - uninstall.bat (卸载程序)
echo    - README_Windows.txt (说明文档)
echo.
echo 🎯 用户只需要:
echo 1. 解压 dist 文件夹到任意位置
echo 2. 双击 "Pokemon GO.bat" 开始游戏
echo.

pause