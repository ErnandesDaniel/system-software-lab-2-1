@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo ========================================
echo  Лабораторная работа #2 — Верификация
echo ========================================
echo.

:: Проверка наличия sqlite3
where sqlite3 >nul 2>nul
if %errorlevel% neq 0 (
    echo [ОШИБКА] sqlite3 не найден.
    echo Установите: winget install SQLite.SQLite
    pause
    exit /b 1
)

set DB=%~dp0ucheb_test.db
set INIT=%~dp0init_abs.sql

:: Удаляем старую БД
if exist "%DB%" del "%DB%"

:: Импорт данных
echo [1/2] Импорт CSV-данных...
sqlite3 "%DB%" < "%INIT%" >nul 2>&1

:: Запросы
echo [2/2] Выполнение запросов...
echo.
sqlite3 -header -column "%DB%" < "%~dp0queries.sql"

echo.
echo ========================================
echo  Готово!
echo ========================================
pause
