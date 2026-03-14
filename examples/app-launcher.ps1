$programToRun = Get-StartApps | Select -ExpandProperty Name | whoamenu -p "Select App:"

(Get-StartApps | ? { $_.Name -like $programToRun }).AppID | % { Start-Process "shell:appsFolder\$_" }