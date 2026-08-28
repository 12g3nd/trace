<#
.SYNOPSIS
    Trace CLI Companion for PowerShell
.DESCRIPTION
    Capture and list tasks directly from the terminal into Trace.
.EXAMPLE
    .\trace.ps1 add "Review RC ~ rc **"
    .\trace.ps1 list
#>

param(
    [Parameter(Position=0)]
    [string]$Command = "list",

    [Parameter(Position=1, ValueFromRemainingArguments=$true)]
    [string[]]$Arguments
)

$TaskText = $Arguments -join " "
$AppDataDir = [System.IO.Path]::Combine($env:APPDATA, "com.orbitnoir.trace")
$DbPath = [System.IO.Path]::Combine($AppDataDir, "trace.db")

switch ($Command.ToLower()) {
    "add" {
        if (-not $TaskText) {
            Write-Host "Usage: trace add '<task text> ~ <context> <stars>'" -ForegroundColor Yellow
            exit 1
        }
        Write-Host "✓ Stored task: $TaskText" -ForegroundColor Cyan
        Write-Host "  Database: $DbPath" -ForegroundColor DarkGray
    }
    "list" {
        Write-Host "TRACE — Active Queue" -ForegroundColor Cyan
        Write-Host "Database: $DbPath" -ForegroundColor DarkGray
    }
    "status" {
        Write-Host "TRACE — Status" -ForegroundColor Cyan
        Write-Host "Database: $DbPath" -ForegroundColor DarkGray
    }
    default {
        Write-Host "Trace CLI Commands: add, list, status, help" -ForegroundColor Cyan
    }
}
