$QuickCopyDir = "$HOME\.local\QuickCopy"

function Send-Notify {
    param (
        [string]$Title,
        [string]$Message,
        [int]$DorationMinutes = 1
    )

    Toast -Text "QuickCopy", $Title, $Message -AppLogo 'C:\fake\path\to\nonexistent.png' -ExpirationTime (Get-Date).AddMinutes($DorationMinutes)
}

if (-not(Test-Path -path $QuickCopyDir)) {
    # Folder doesn't exists, create the folder
    New-Item -Path $QuickCopyDir -ItemType Directory
}

$files = Get-ChildItem "$QuickCopyDir" -File | ForEach-Object { (Get-Item $_.FullName).BaseName }
$files = @() + $files + @('*New File*') #Empty array fist to prevent string concatination
$file = $files | whoamenu.exe
if($file -eq ""){
	Exit 1
}

if($file -eq '*New File*') {
    $newFile = whoamenu -p "File Name"
	if($newFile -eq ""){
		Exit 1
	}
    $fullPath = [IO.Path]::Combine($QuickCopyDir, "$newFile.txt") 
    New-Item -Path $fullPath -ItemType File
    $file = $newFile
    Send-Notify "Added File" $fullPath
}

$fullItemsPath = [IO.Path]::Combine($QuickCopyDir, "$file.txt")
$items = Get-Content $fullItemsPath
$items = @() + $items + @('*New Item*')

$item = $items | whoamenu -p "Item"
if($item -eq ""){
	Exit 1
}

if($item -eq '*New Item*') {
    $item = whoamenu -p "New Item"
	if($item -eq ""){
		Exit 1
	}
    Add-Content -Path $fullItemsPath -Value $item
    Send-Notify -Title "Added Item $item" -Message "Added to: $$fileItemsPath"
}

Set-Clipboard -Value $item
