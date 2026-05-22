@echo off
powershell -Command "Get-Process -Name java -ErrorAction SilentlyContinue | Stop-Process -Force"
