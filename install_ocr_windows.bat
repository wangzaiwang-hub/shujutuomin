@echo off
chcp 65001 >nul
echo ========================================
echo CheersAI Vault - OCR 环境自动安装脚本
echo ========================================
echo.

REM 检查是否以管理员权限运行
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [错误] 请以管理员权限运行此脚本
    echo 右键点击脚本，选择"以管理员身份运行"
    pause
    exit /b 1
)

echo [1/4] 检查 Python 环境...
python --version >nul 2>&1
if %errorLevel% neq 0 (
    echo [警告] 未检测到 Python 环境
    echo 请先安装 Python 3.7 或更高版本
    echo 下载地址: https://www.python.org/downloads/
    echo.
    echo 安装时请勾选 "Add Python to PATH"
    pause
    exit /b 1
) else (
    python --version
    echo [✓] Python 已安装
)
echo.

echo [2/4] 安装 Python 依赖包...
echo 正在安装: pdf2image, pytesseract, Pillow
pip install pdf2image pytesseract Pillow
if %errorLevel% neq 0 (
    echo [错误] Python 包安装失败
    pause
    exit /b 1
)
echo [✓] Python 包安装完成
echo.

echo [3/4] 检查 Tesseract OCR...
tesseract --version >nul 2>&1
if %errorLevel% neq 0 (
    echo [警告] 未检测到 Tesseract OCR
    echo.
    echo 请手动安装 Tesseract OCR:
    echo 1. 访问: https://github.com/UB-Mannheim/tesseract/wiki
    echo 2. 下载最新版本的安装包
    echo 3. 安装时选择包含中文语言包 (chi_sim)
    echo 4. 安装完成后重新运行此脚本
    echo.
    pause
    exit /b 1
) else (
    tesseract --version
    echo [✓] Tesseract OCR 已安装
)
echo.

echo [4/4] 检查 Poppler (pdf2image 依赖)...
where pdfinfo >nul 2>&1
if %errorLevel% neq 0 (
    echo [警告] 未检测到 Poppler
    echo.
    echo 请手动安装 Poppler:
    echo 1. 访问: https://github.com/oschwartz10612/poppler-windows/releases
    echo 2. 下载最新版本 (poppler-xx.xx.x-0.zip)
    echo 3. 解压到任意目录 (如 C:\Program Files\poppler)
    echo 4. 将 bin 目录添加到系统 PATH 环境变量
    echo    例如: C:\Program Files\poppler\Library\bin
    echo 5. 重启命令提示符后重新运行此脚本
    echo.
    pause
    exit /b 1
) else (
    pdfinfo -v
    echo [✓] Poppler 已安装
)
echo.

echo ========================================
echo [✓] OCR 环境安装完成！
echo ========================================
echo.
echo 现在可以使用 CheersAI Vault 处理扫描版 PDF 了
echo.
pause
