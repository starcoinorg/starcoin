$diff=$(git diff)
Write-Host "$diff"

$changed_files=$(git status --porcelain)
Write-Host "$changed_files"
