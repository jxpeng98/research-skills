param(
    [string]$Profile = "",

    [ValidateSet("codex", "claude", "gemini", "antigravity", "all")]
    [string]$Target = "all",

    [string]$ProjectDir = (Get-Location).Path,
    [string]$Repo = "jxpeng98/research-skills",
    [string]$SourceRepo = "",
    [string]$Ref = "",
    [Alias("Prerelease")]
    [switch]$Beta,

    [ValidateSet("tag", "branch", "local")]
    [string]$RefType = "tag",

    [switch]$Overwrite,
    [switch]$InstallCli,
    [switch]$NoCli,
    [switch]$Doctor,
    [switch]$NoDoctor,
    [string]$CliDir = "",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw "bootstrap_research_skill.ps1 requires PowerShell 7+. Install it with: winget install --id Microsoft.PowerShell --source winget, then rerun with pwsh."
}

function Write-Info([string]$Message) {
    Write-Host "  $Message"
}

function Write-Warn([string]$Message) {
    Write-Host "  [warn] $Message"
}

function Write-Ok([string]$Label, [string]$Message) {
    Write-Host ("  [ok]   {0,-12} -> {1}" -f $Label, $Message)
}

function Write-Skip([string]$Label, [string]$Message) {
    Write-Host ("  [skip] {0,-12} -> {1}" -f $Label, $Message)
}

function Invoke-NativeChecked([string]$ExePath, [string[]]$Arguments) {
    & $ExePath @Arguments 2>&1 | Out-Host
    if ($LASTEXITCODE -ne 0) {
        $argLine = if ($Arguments) { " " + ($Arguments -join " ") } else { "" }
        throw "Command failed: $ExePath$argLine"
    }
}

function Ensure-PathEntry([string]$EntryPath, [switch]$PersistUser) {
    if (-not $EntryPath) {
        return
    }

    $normalizedEntry = [System.IO.Path]::GetFullPath($EntryPath).TrimEnd('\')
    $pathEntries = @($env:Path -split ';' | Where-Object { $_ })
    $hasCurrent = $false
    foreach ($entry in $pathEntries) {
        try {
            if ([System.IO.Path]::GetFullPath($entry).TrimEnd('\') -ieq $normalizedEntry) {
                $hasCurrent = $true
                break
            }
        }
        catch {
        }
    }
    if (-not $hasCurrent) {
        $env:Path = if ($env:Path) { "$normalizedEntry;$env:Path" } else { $normalizedEntry }
    }

    if (-not $PersistUser) {
        return
    }

    $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    $userEntries = @($userPath -split ';' | Where-Object { $_ })
    foreach ($entry in $userEntries) {
        try {
            if ([System.IO.Path]::GetFullPath($entry).TrimEnd('\') -ieq $normalizedEntry) {
                return
            }
        }
        catch {
        }
    }
    $newUserPath = if ($userPath) { "$normalizedEntry;$userPath" } else { $normalizedEntry }
    [System.Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
}

function Show-ProfileHelp {
    Write-Host ""
    Write-Host "Choose an install profile:"
    Write-Host ""
    Write-Host "  1) partial"
    Write-Host "     - Installs workflow assets and project integration files only"
    Write-Host "     - Does not install shell CLI"
    Write-Host "     - Does not require Python"
    Write-Host "     - Does not run orchestrator doctor"
    Write-Host ""
    Write-Host "  2) full"
    Write-Host "     - Installs workflow assets and project integration files"
    Write-Host "     - Installs shell CLI wrappers on Windows"
    Write-Host "     - Ensures Python 3.12 is available via mise if missing"
    Write-Host "     - Runs orchestrator doctor"
    Write-Host ""
}

function Resolve-Profile([string]$CurrentProfile) {
    if ($CurrentProfile) {
        $normalized = $CurrentProfile.Trim().ToLowerInvariant()
        if ($normalized -in @("partial", "full")) {
            return $normalized
        }
        throw "Unsupported profile: $CurrentProfile. Use partial or full."
    }

    Show-ProfileHelp
    while ($true) {
        $answer = Read-Host "Select profile [1/2]"
        switch -Regex ($answer.Trim().ToLowerInvariant()) {
            '^(1|partial)$' { return "partial" }
            '^(2|full)$' { return "full" }
            default { Write-Host "Please enter 1, 2, partial, or full." }
        }
    }
}

function Normalize-Repo([string]$RawRepo) {
    $value = ""
    if ($null -ne $RawRepo) {
        $value = $RawRepo.Trim()
    }
    if (-not $value) {
        throw "empty repo spec"
    }
    if ($value -match '^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$') {
        return $value
    }
    $value = $value -replace '^git@[^:]+:', '' `
                     -replace '^ssh://git@[^/]+/', '' `
                     -replace '^https?://[^/]+/', '' `
                     -replace '\.git$', ''
    $value = $value.TrimStart('/')
    $parts = $value.Split('/')
    if ($parts.Length -ge 2) {
        $candidate = "$($parts[0])/$($parts[1])"
        if ($candidate -match '^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$') {
            return $candidate
        }
    }
    throw "unsupported repo spec: $RawRepo"
}

function Resolve-LocalPath([string]$RawPath) {
    if (-not $RawPath) {
        throw "empty local path"
    }
    return [System.IO.Path]::GetFullPath($RawPath)
}

function Get-VersionSortKey([string]$Tag, [bool]$RequireBeta = $false) {
    $match = [regex]::Match($Tag, '^v?(\d+)\.(\d+)\.(\d+)(?:-beta\.(\d+)|b(\d+))?$')
    if (-not $match.Success) {
        return $null
    }
    $betaRaw = ""
    if ($match.Groups[4].Success) {
        $betaRaw = $match.Groups[4].Value
    }
    elseif ($match.Groups[5].Success) {
        $betaRaw = $match.Groups[5].Value
    }
    if ($RequireBeta -and -not $betaRaw) {
        return $null
    }
    $betaKey = if ($betaRaw) { [int]$betaRaw } else { 1000000000 }
    return "{0:D10}.{1:D10}.{2:D10}.{3:D10}" -f [int]$match.Groups[1].Value, [int]$match.Groups[2].Value, [int]$match.Groups[3].Value, $betaKey
}

function Select-LatestVersionTag([object[]]$Tags, [bool]$RequireBeta = $false) {
    $bestTag = $null
    $bestKey = $null
    foreach ($candidate in $Tags) {
        if ($null -eq $candidate) {
            continue
        }
        $tag = [string]$candidate
        if (-not $tag) {
            continue
        }
        $key = Get-VersionSortKey $tag $RequireBeta
        if (-not $key) {
            continue
        }
        if (-not $bestKey -or $key -gt $bestKey) {
            $bestKey = $key
            $bestTag = $tag
        }
    }
    return $bestTag
}

function Resolve-LatestTag([string]$RepoSpec, [switch]$RequireBeta) {
    if (-not $RequireBeta) {
        $apiUrl = "https://api.github.com/repos/$RepoSpec/releases/latest"
        try {
            $response = Invoke-RestMethod -Uri $apiUrl
            if ($response.tag_name) {
                return [string]$response.tag_name
            }
        }
        catch {
        }
    }

    try {
        $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$RepoSpec/releases?per_page=20"
        $releaseTags = @($releases | ForEach-Object { [string]$_.tag_name })
        $resolved = Select-LatestVersionTag $releaseTags $RequireBeta
        if ($resolved) {
            return $resolved
        }
    }
    catch {
    }

    try {
        $tags = Invoke-RestMethod -Uri "https://api.github.com/repos/$RepoSpec/tags?per_page=50"
        $tagNames = @($tags | ForEach-Object { [string]$_.name })
        $resolved = Select-LatestVersionTag $tagNames $RequireBeta
        if ($resolved) {
            return $resolved
        }
    }
    catch {
    }

    if ($RequireBeta) {
        throw "Unable to resolve latest beta/prerelease tag for $RepoSpec"
    }
    throw "Unable to resolve latest release tag for $RepoSpec"
}

function Get-ArchiveUrl([string]$RepoSpec, [string]$ResolvedRef, [string]$ResolvedRefType) {
    if ($ResolvedRefType -eq "tag") {
        return "https://github.com/$RepoSpec/archive/refs/tags/$ResolvedRef.zip"
    }
    return "https://github.com/$RepoSpec/archive/refs/heads/$ResolvedRef.zip"
}

function Find-Bash {
    $command = Get-Command bash -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }
    $candidates = @(
        "C:\Program Files\Git\bin\bash.exe",
        "C:\Program Files\Git\usr\bin\bash.exe",
        "${env:ProgramFiles}\Git\bin\bash.exe",
        "${env:ProgramFiles}\Git\usr\bin\bash.exe"
    ) | Where-Object { $_ -and (Test-Path $_) }
    if ($candidates.Count -gt 0) {
        return $candidates[0]
    }
    return $null
}

function Ensure-GitBash {
    $bash = Find-Bash
    if ($bash) {
        return $bash
    }
    $winget = Get-Command winget -ErrorAction SilentlyContinue
    if (-not $winget) {
        throw "Git Bash is required for the Windows shell CLI wrappers, but winget was not found."
    }
    Write-Info "Git Bash not found. Installing Git for Windows via winget..."
    if ($DryRun) {
        Write-Host '[dry-run] winget install -e --id Git.Git --source winget'
        return "C:\Program Files\Git\bin\bash.exe"
    }
    Invoke-NativeChecked $winget.Source @("install", "-e", "--id", "Git.Git", "--source", "winget")
    $bash = Find-Bash
    if (-not $bash) {
        throw "Git for Windows installation completed but bash.exe was not found."
    }
    return $bash
}

function Find-Mise {
    $command = Get-Command mise -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }
    $candidates = @(
        "$env:LOCALAPPDATA\mise\bin\mise.exe",
        "$env:USERPROFILE\.local\bin\mise.exe"
    ) | Where-Object { $_ -and (Test-Path $_) }
    if ($candidates.Count -gt 0) {
        return $candidates[0]
    }
    return $null
}

function Ensure-Mise {
    $mise = Find-Mise
    if ($mise) {
        Ensure-PathEntry (Split-Path -Parent $mise) -PersistUser
        return $mise
    }
    $winget = Get-Command winget -ErrorAction SilentlyContinue
    if (-not $winget) {
        throw "mise is required for full install, but winget was not found. Install mise manually or choose partial."
    }
    Write-Info "mise not found. Installing mise via winget..."
    if ($DryRun) {
        Write-Host '[dry-run] winget install jdx.mise'
        return "$env:LOCALAPPDATA\mise\bin\mise.exe"
    }
    Invoke-NativeChecked $winget.Source @("install", "jdx.mise")
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    $mise = Find-Mise
    if (-not $mise) {
        throw "mise installation completed but mise.exe was not found."
    }
    Ensure-PathEntry (Split-Path -Parent $mise) -PersistUser
    return $mise
}

function Ensure-PythonRuntime {
    $python = Get-Command python3 -ErrorAction SilentlyContinue
    if ($python) {
        try {
            $version = & $python.Source -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')"
            $parts = $version.Trim().Split(".")
            if ([int]$parts[0] -gt 3 -or ([int]$parts[0] -eq 3 -and [int]$parts[1] -ge 12)) {
                Write-Info "python:  $($python.Source) ($version)"
                return @{
                    Mode = "direct"
                    Python = $python.Source
                    Mise = $null
                }
            }
        }
        catch {
        }
        Write-Info "python3 exists but is below 3.12. Installing python@3.12 via mise..."
    }
    else {
        Write-Info "python3 not found. Full install will add python@3.12 via mise..."
    }

    $mise = Ensure-Mise
    if ($DryRun) {
        Write-Host '[dry-run] mise install python@3.12'
        Write-Host '[dry-run] mise use -g python@3.12'
        return @{
            Mode = "mise"
            Python = "python"
            Mise = $mise
        }
    }

    Invoke-NativeChecked $mise @("install", "python@3.12")
    Invoke-NativeChecked $mise @("use", "-g", "python@3.12")
    return @{
        Mode = "mise"
        Python = "python"
        Mise = $mise
    }
}

function Ensure-Dir([string]$PathValue) {
    if ($DryRun) {
        return
    }
    New-Item -ItemType Directory -Path $PathValue -Force | Out-Null
}

function Remove-PathIfNeeded([string]$PathValue) {
    if ($DryRun) {
        return
    }
    if (Test-Path $PathValue) {
        Remove-Item -Path $PathValue -Recurse -Force
    }
}

function Copy-InstallItem([string]$Source, [string]$Destination, [string]$Label, [switch]$ForceCopy) {
    $resolvedSource = [System.IO.Path]::GetFullPath($Source)
    $resolvedDestination = [System.IO.Path]::GetFullPath($Destination)
    if ($resolvedSource -eq $resolvedDestination) {
        Write-Skip $Label "$Destination (same path)"
        return
    }
    if ((Test-Path $Destination) -and -not $ForceCopy -and -not $Overwrite) {
        Write-Skip $Label "$Destination (use --overwrite)"
        return
    }
    if ((Test-Path $Destination) -and ($ForceCopy -or $Overwrite)) {
        Remove-PathIfNeeded $Destination
    }
    Ensure-Dir ([System.IO.Path]::GetDirectoryName($Destination))
    if ($DryRun) {
        Write-Ok $Label $Destination
        return
    }
    Copy-Item -Path $Source -Destination $Destination -Recurse -Force
    Write-Ok $Label $Destination
}

function Write-CmdWrapper([string]$Destination, [string]$BashPath, [string]$ScriptName) {
    $content = @(
        '@echo off'
        ('"{0}" "%~dp0{1}" %*' -f $BashPath, $ScriptName)
    ) -join "`r`n"
    if ($DryRun) {
        Write-Ok "Wrapper" $Destination
        return
    }
    Set-Content -Path $Destination -Value $content -Encoding ASCII
    Write-Ok "Wrapper" $Destination
}

function Install-ShellCliWindows([string]$RepoRoot, [string]$CliRoot, [string]$BashPath) {
    Write-Host ""
    Write-Host "== Shell CLI =="
    $cliScript = Join-Path $RepoRoot "scripts\research_skills_cli.sh"
    $bootstrapScript = Join-Path $RepoRoot "scripts\bootstrap_research_skill.sh"
    $cliDest = Join-Path $CliRoot "research-skills"
    $bootstrapDest = Join-Path $CliRoot "research-skills-bootstrap"
    Ensure-Dir $CliRoot
    Copy-InstallItem $cliScript $cliDest "CLI" -ForceCopy
    Copy-InstallItem $bootstrapScript $bootstrapDest "Bootstrap" -ForceCopy
    Write-CmdWrapper (Join-Path $CliRoot "research-skills.cmd") $BashPath "research-skills"
    Write-CmdWrapper (Join-Path $CliRoot "rsk.cmd") $BashPath "research-skills"
    Write-CmdWrapper (Join-Path $CliRoot "rsw.cmd") $BashPath "research-skills"
    if (($env:Path -split ';') -contains $CliRoot) {
        Write-Info "cli dir on PATH: $CliRoot"
    }
    else {
        Write-Warn "CLI installed to $CliRoot but this directory is not on PATH."
    }
}

function Write-QuickstartFallback([string]$QuickstartPath) {
    $content = @(
        '# Research Skills for Gemini Runtime'
        ''
        'Use this project through orchestrator for Codex/Claude/Gemini collaboration:'
        ''
        '```bash'
        'python3 -m bridges.orchestrator doctor --cwd .'
        'python3 -m bridges.orchestrator parallel --prompt "Analyze this study design" --cwd . --summarizer gemini'
        'python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic your-topic --cwd . --triad'
        '```'
    ) -join "`r`n"
    if ($DryRun) {
        Write-Ok "Quickstart" $QuickstartPath
        return
    }
    Ensure-Dir ([System.IO.Path]::GetDirectoryName($QuickstartPath))
    Set-Content -Path $QuickstartPath -Value $content -Encoding UTF8
    Write-Ok "Quickstart" $QuickstartPath
}

function Read-InstallManifest([string]$RepoRoot) {
    $manifestPath = Join-Path $RepoRoot "install\install_manifest.tsv"
    $entries = @()
    foreach ($line in Get-Content $manifestPath) {
        if (-not $line.Trim() -or $line.TrimStart().StartsWith("#")) {
            continue
        }
        $parts = $line -split "`t"
        if ($parts.Length -lt 5) {
            continue
        }
        $entries += [pscustomobject]@{
            Target = $parts[0]
            Operation = $parts[1]
            Label = $parts[2]
            Source = $parts[3]
            Destination = $parts[4]
        }
    }
    return $entries
}

function Expand-ManifestPath([string]$Template, [hashtable]$Values) {
    $result = $Template
    foreach ($key in $Values.Keys) {
        $result = $result.Replace('${' + $key + '}', [string]$Values[$key])
    }
    return $result
}

function Install-FromRepo([string]$RepoRoot, [string]$ProjectRoot, [string]$InstallTarget, [bool]$DoInstallCli, [bool]$DoDoctor, [hashtable]$PythonRuntime) {
    $projectRoot = [System.IO.Path]::GetFullPath($ProjectRoot)
    $skillSrc = Join-Path $RepoRoot "research-paper-workflow"
    if (-not (Test-Path (Join-Path $skillSrc "SKILL.md"))) {
        throw "Missing skill source: $skillSrc\SKILL.md"
    }

    $codexHome = if ($env:CODEX_HOME) { $env:CODEX_HOME } else { Join-Path $env:USERPROFILE ".codex" }
    $claudeHome = if ($env:CLAUDE_CODE_HOME) { $env:CLAUDE_CODE_HOME } else { Join-Path $env:USERPROFILE ".claude" }
    $geminiHome = if ($env:GEMINI_HOME) { $env:GEMINI_HOME } else { Join-Path $env:USERPROFILE ".gemini" }
    $antigravityHome = if ($env:ANTIGRAVITY_HOME) { $env:ANTIGRAVITY_HOME } else { Join-Path $env:USERPROFILE ".gemini\antigravity" }
    $cliRoot = if ($CliDir) { $CliDir } else { Join-Path $env:USERPROFILE ".local\bin" }
    $manifest = Read-InstallManifest $RepoRoot
    $manifestValues = @{
        PROJECT_DIR = $projectRoot
        CODEX_HOME = $codexHome
        CLAUDE_CODE_HOME = $claudeHome
        GEMINI_HOME = $geminiHome
        ANTIGRAVITY_HOME = $antigravityHome
    }

    Write-Host ""
    Write-Host "Research Skills Windows Bootstrap"
    Write-Info "source:  $RepoRoot"
    Write-Info "project: $projectRoot"
    Write-Info "target:  $InstallTarget"
    Write-Info "profile: $Profile"

    Write-Host ""
    Write-Host "== CLI Checks =="
    $targets = if ($InstallTarget -eq "all") { @("codex", "claude", "gemini", "antigravity") } else { @($InstallTarget) }
    foreach ($item in $targets) {
        $resolved = Get-Command $item -ErrorAction SilentlyContinue
        if ($resolved) {
            Write-Ok "CLI" "$item -> $($resolved.Source)"
        }
        else {
            Write-Skip "CLI" "$item -> missing"
        }
    }

    foreach ($sectionTarget in @("codex", "claude", "gemini", "antigravity")) {
        if ($InstallTarget -ne "all" -and $InstallTarget -ne $sectionTarget) {
            continue
        }
        Write-Host ""
        if ($sectionTarget -eq "antigravity") {
            Write-Host "== Antigravity =="
        }
        else {
            Write-Host "== $([char]::ToUpper($sectionTarget[0]) + $sectionTarget.Substring(1)) =="
        }
        foreach ($entry in $manifest | Where-Object { $_.Target -eq $sectionTarget }) {
            $src = Join-Path $RepoRoot $entry.Source
            $dest = Expand-ManifestPath $entry.Destination $manifestValues
            switch ($entry.Operation) {
                "dir-copy" { Copy-InstallItem $src $dest $entry.Label }
                "file-copy" { Copy-InstallItem $src $dest $entry.Label }
                "glob-copy" {
                    $workflowDest = $dest.TrimEnd('\','/')
                    Ensure-Dir $workflowDest
                    $workflowFiles = Get-ChildItem -Path (Join-Path $RepoRoot ".agent\workflows") -Filter *.md | Sort-Object Name
                    foreach ($workflow in $workflowFiles) {
                        Copy-InstallItem $workflow.FullName (Join-Path $workflowDest $workflow.Name) "Workflow"
                    }
                }
                "claude-template" {
                    $claudeDest = $dest
                    if ((Test-Path $claudeDest) -and -not $Overwrite) {
                        $claudeDest = Join-Path $projectRoot "CLAUDE.research-skills.md"
                    }
                    Copy-InstallItem $src $claudeDest $entry.Label
                }
                "quickstart-file" {
                    if (Test-Path $src) {
                        Copy-InstallItem $src $dest $entry.Label
                    }
                    else {
                        Write-QuickstartFallback $dest
                    }
                }
                "conditional-dir-copy" {
                    $ag = Get-Command antigravity -ErrorAction SilentlyContinue
                    if ($ag) {
                        Copy-InstallItem $src $dest $entry.Label
                    }
                    else {
                        Write-Skip $entry.Label ($dest + " (antigravity CLI not found)")
                    }
                }
                default {
                    throw "Unsupported manifest operation: $($entry.Operation)"
                }
            }
        }
    }

    if ($DoInstallCli) {
        $bashPath = Ensure-GitBash
        Install-ShellCliWindows $RepoRoot $cliRoot $bashPath
    }

    Write-Host ""
    Write-Host "== Project Env =="
    foreach ($entry in $manifest | Where-Object { $_.Target -eq "project" }) {
        Copy-InstallItem (Join-Path $RepoRoot $entry.Source) (Expand-ManifestPath $entry.Destination $manifestValues) $entry.Label
    }

    if ($DoDoctor -and $PythonRuntime) {
        Write-Host ""
        Write-Host "== Doctor =="
        if ($DryRun) {
            Write-Ok "Doctor" "dry-run ($projectRoot)"
        }
        else {
            Push-Location $RepoRoot
            try {
                $previousPythonPath = $env:PYTHONPATH
                if ([string]::IsNullOrWhiteSpace($previousPythonPath)) {
                    $env:PYTHONPATH = $RepoRoot
                }
                else {
                    $env:PYTHONPATH = "$RepoRoot$([System.IO.Path]::PathSeparator)$previousPythonPath"
                }
                if ($PythonRuntime.Mode -eq "mise") {
                    & $PythonRuntime.Mise exec python@3.12 -- python -m bridges.orchestrator doctor --cwd $projectRoot
                }
                else {
                    & $PythonRuntime.Python -m bridges.orchestrator doctor --cwd $projectRoot
                }
            }
            finally {
                if ($null -eq $previousPythonPath) {
                    Remove-Item Env:PYTHONPATH -ErrorAction SilentlyContinue
                }
                else {
                    $env:PYTHONPATH = $previousPythonPath
                }
                Pop-Location
            }
        }
    }

    Write-Host ""
    Write-Host "[done] Installation complete"
    if ($DoInstallCli) {
        Write-Host "       Add $cliRoot to PATH to use research-skills / rsk / rsw on Windows."
    }
    Write-Host "       Restart Codex / Claude Code / Gemini CLI to activate changes."
}

$Profile = Resolve-Profile $Profile
$sourceRepoRoot = $null
if ($SourceRepo) {
    $sourceRepoRoot = Resolve-LocalPath $SourceRepo
    if (-not (Test-Path (Join-Path $sourceRepoRoot "scripts\bootstrap_research_skill.ps1"))) {
        throw "Local source repo is missing scripts\bootstrap_research_skill.ps1: $sourceRepoRoot"
    }
    $Repo = "<local>"
}
else {
    $Repo = Normalize-Repo $Repo
}

$runDoctor = $false
$installCliByProfile = $false
if ($Profile -eq "full") {
    $runDoctor = $true
    $installCliByProfile = $true
}
if ($Doctor) {
    $runDoctor = $true
}
if ($NoDoctor) {
    $runDoctor = $false
}
if ($InstallCli) {
    $installCliByProfile = $true
}
if ($NoCli) {
    $installCliByProfile = $false
}

$pythonRuntime = $null
if ($Profile -eq "full") {
    $pythonRuntime = Ensure-PythonRuntime
}

$resolvedRef = ""
$archiveUrl = ""
$tmpDir = $null
$archivePath = $null
$extractRoot = $null
if ($sourceRepoRoot) {
    $resolvedRef = "<checkout>"
    $RefType = "local"
    $archiveUrl = "<local-checkout>"
}
else {
    $resolvedRef = if ($Ref) { $Ref } else { Resolve-LatestTag $Repo -RequireBeta:$Beta }
    $archiveUrl = Get-ArchiveUrl $Repo $resolvedRef $RefType
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("research-skills-bootstrap-" + [System.Guid]::NewGuid().ToString("N"))
    $archivePath = Join-Path $tmpDir "research-skills.zip"
    $extractRoot = Join-Path $tmpDir "src"
    $null = New-Item -ItemType Directory -Path $extractRoot -Force
}

try {
    Write-Host ""
    Write-Host "Research Skills Windows Bootstrap"
    Write-Info "repo:    $Repo"
    Write-Info "ref:     $resolvedRef ($RefType)"
    Write-Info "project: $ProjectDir"
    Write-Info "target:  $Target"
    Write-Info "profile: $Profile"
    Write-Info "archive: $archiveUrl"
    if ($sourceRepoRoot) {
        Write-Info "source:  $sourceRepoRoot"
    }

    if ($DryRun) {
        if ($sourceRepoRoot) {
            Write-Host "[dry-run] Use local checkout at $sourceRepoRoot"
        }
        else {
            Write-Host "[dry-run] Invoke-WebRequest -Uri $archiveUrl -OutFile $archivePath"
            Write-Host "[dry-run] Expand-Archive -Path $archivePath -DestinationPath $extractRoot"
        }
        Write-Host "[dry-run] Install workflow assets into client directories for target '$Target'"
        if ($installCliByProfile) {
            $cliRoot = if ($CliDir) { $CliDir } else { Join-Path $env:USERPROFILE ".local\bin" }
            Write-Host "[dry-run] Install shell CLI wrappers into $cliRoot"
        }
        if ($runDoctor) {
            Write-Host "[dry-run] Run orchestrator doctor after install"
        }
        exit 0
    }

    if ($sourceRepoRoot) {
        $repoRoot = Get-Item -Path $sourceRepoRoot
    }
    else {
        Invoke-WebRequest -Uri $archiveUrl -OutFile $archivePath
        Expand-Archive -Path $archivePath -DestinationPath $extractRoot -Force
        $repoRoot = Get-ChildItem -Path $extractRoot -Directory | Select-Object -First 1
        if (-not $repoRoot) {
            throw "Failed to locate extracted repository root."
        }
    }
    Install-FromRepo $repoRoot.FullName $ProjectDir $Target $installCliByProfile $runDoctor $pythonRuntime
}
finally {
    if ($tmpDir -and (Test-Path $tmpDir)) {
        Remove-Item -Path $tmpDir -Recurse -Force
    }
}
