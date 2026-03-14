$PROJECTFOLDER = "$HOME\workspace"

$projectsDirs = Get-ChildItem "$PROJECTFOLDER" -Directory | ForEach-Object { $_.Name }

$proj = $projectsDirs | Sort-Object | Out-String -Stream | whoamenu -p "Code Project"

if (-not [string]::IsNullOrWhiteSpace($proj) ) {
    Set-Location -Path "$PROJECTFOLDER\$proj"
    code .
    wt -d .
}
