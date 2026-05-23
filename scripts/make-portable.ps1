# 生成绿色免安装版（便携版）
# 用法: powershell -File scripts/make-portable.ps1

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$releaseDir = Join-Path $projectRoot "src-tauri\target\release"
$exePath = Join-Path $releaseDir "system_toolbox_tauri.exe"
$resourcesSrc = Join-Path $projectRoot "src-tauri\resources"

if (-not (Test-Path $exePath)) {
    Write-Host "错误: 请先运行 npm run build 生成 release exe" -ForegroundColor Red
    exit 1
}

# 输出目录
$portableDir = Join-Path $releaseDir "portable\系统工具箱"
if (Test-Path $portableDir) {
    Remove-Item $portableDir -Recurse -Force
}
New-Item -ItemType Directory -Path $portableDir -Force | Out-Null

# 复制主程序
Copy-Item $exePath $portableDir

# 复制资源文件
$destResources = Join-Path $portableDir "resources"
Copy-Item $resourcesSrc $destResources -Recurse

Write-Host ""
Write-Host "=== 绿色便携版生成完成 ===" -ForegroundColor Green
Write-Host "目录: $portableDir"
Write-Host ""
Write-Host "文件结构:"
Get-ChildItem $portableDir -Recurse | ForEach-Object {
    $rel = $_.FullName.Replace($portableDir, "").TrimStart("\")
    if ($_.PSIsContainer) { Write-Host "  [目录] $rel" } else { Write-Host "  $rel ($([math]::Round($_.Length/1KB, 1)) KB)" }
}
Write-Host ""
Write-Host "注意: 目标电脑需要已安装 WebView2 Runtime（Win10 21H2+/Win11 自带）" -ForegroundColor Yellow
Write-Host "如目标电脑没有，可将 WebView2 引导安装程序放入文件夹，让用户先运行。" -ForegroundColor Yellow
