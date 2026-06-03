@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo ========================================
echo  Lab 2 — SQLite Verification
echo ========================================
echo.

:: Check sqlite3 availability
where sqlite3 >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] sqlite3 not found.
    echo Install: winget install SQLite.SQLite
    pause
    exit /b 1
)

set DB=%~dp0ucheb_test.db
set INIT=%~dp0init_abs.sql

:: Delete old DB
if exist "%DB%" del "%DB%"

:: Import data
echo [1/2] Importing CSV data...
sqlite3 "%DB%" < "%INIT%" >nul 2>&1

:: Run queries
echo [2/2] Running queries...
echo.
sqlite3 -header -column "%DB%" < "%~dp0queries.sql"

echo.
echo ========================================
echo  Done!
echo ========================================
pause
