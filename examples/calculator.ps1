# Requires the BurntToast powershell module

$calc = whoamenu.exe -p " Enter Calculation: "
try {
    $encoded = [Uri]::EscapeDataString($calc)
    $result = Invoke-RestMethod -Uri "http://api.mathjs.org/v4/?expr=$encoded" 
} catch {
    $result = "Somen' went wrong boss"
    Write-Host "An error occurred: $($_.Exception.Message)"
}


Toast -Text "Calculation", "Calc: $calc", "Result: $result" -AppLogo 'C:\fake\path\to\nonexistent.png' -ExpirationTime (Get-Date).AddMinutes(1)