@echo off
REM 在项目根目录执行初始化（需先 docker compose up -d，且不依赖 PowerShell 执行策略）
setlocal
cd /d "%~dp0.."

echo [init-db] 导入 migrations\001_init.sql ...
docker compose exec -T mysql mysql -uroot -phnudoc_dev hnudoc < migrations\001_init.sql
if errorlevel 1 (
  echo [init-db] 失败：请先在本目录执行 docker compose up -d，并确认 Docker Desktop 已启动。
  exit /b 1
)
echo [init-db] 完成。
exit /b 0
