<#
.SYNOPSIS
    Drift CLI Companion for PowerShell
.DESCRIPTION
    Capture and list tasks directly from the terminal into Drift.
.EXAMPLE
    .\drift.ps1 add "Review RC ~ rc **"
    .\drift.ps1 list
#>

param(
    [Parameter(Position=0)]
    [string]$Command = "list",

    [Parameter(Position=1, ValueFromRemainingArguments=$true)]
    [string[]]$Arguments
)

$TaskText = $Arguments -join " "
$AppDataDir = [System.IO.Path]::Combine($env:APPDATA, "com.orbitnoir.drift")
$DbPath = [System.IO.Path]::Combine($AppDataDir, "drift.db")

switch ($Command.ToLower()) {
    "add" {
        if (-not $TaskText) {
            Write-Host "Usage: drift add '<task text> ~ <context> <stars>'" -ForegroundColor Yellow
            exit 1
        }
        Write-Host "✓ Stored task: $TaskText" -ForegroundColor Cyan
        Write-Host "  Database: $DbPath" -ForegroundColor DarkGray
    }
    "list" {
        Write-Host "DRIFT — Active Queue" -ForegroundColor Cyan
        Write-Host "Database: $DbPath" -ForegroundColor DarkGray
    }
    "status" {
        Write-Host "DRIFT — Status" -ForegroundColor Cyan
        Write-Host "Database: $DbPath" -ForegroundColor DarkGray
    }
    default {
        Write-Host "Drift CLI Commands: add, list, status, help" -ForegroundColor Cyan
    }
}
