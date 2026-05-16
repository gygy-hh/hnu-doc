# 在 Docker MySQL 就绪后初始化表结构（在项目根目录执行）
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$sqlPath = Join-Path $root "migrations\001_init.sql"
if (-not (Test-Path $sqlPath)) {
    Write-Error "找不到迁移文件: $sqlPath"
}

$sql = Get-Content -LiteralPath $sqlPath -Raw -Encoding UTF8
$sql | docker compose exec -T mysql mysql -uroot -phnudoc_dev hnudoc
if ($LASTEXITCODE -ne 0) {
    Write-Error "数据库初始化失败。请先执行: docker compose up -d"
}
Write-Host "hnudoc 库结构已导入。"
