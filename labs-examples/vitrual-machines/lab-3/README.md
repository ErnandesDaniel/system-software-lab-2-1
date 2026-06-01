# lab-3: Mark-and-Sweep Garbage Collector

GC написан на MyLang, компилируется в оба таргета (NASM и JVM).

## Управление GC

Глобальная переменная `gc_enabled = 1` включает сборщик, `gc_enabled = 0` отключает.

В `demo_gc_on()`: `gc_enabled = 1`, GC запускается каждые 10 итераций.
В `demo_gc_off()`: `gc_enabled = 0`, GC не запускается — память только растёт.

## Компиляция и запуск

### NASM

```powershell
cargo run -- labs-examples/vitrual-machines/lab-3/input.mylang -o output -t nasm
.\output\program.exe
```

Без GC (GC_OFF) программа падает на ~2000 итерациях (~2 GB) с access violation (exit code `-1073741819` или `0xC0000005`).

Проверить: `.\output\program.exe; Write-Output "Exit: $LASTEXITCODE"`

### JVM

```powershell
cargo run -- labs-examples/vitrual-machines/lab-3/input.mylang -o output -t jvm
java -cp output Main
```

JVM имеет встроенный GC, поэтому проходит все 5000 итераций без ошибок.

## Сравнение таргетов

| | NASM | JVM |
|---|---|---|
| GC_ON | ~30 MB, 5 GC прогонов | ~30 MB, 5 GC прогонов |
| GC_OFF | OOM на ~2000 итерациях | Все 5000 итераций без ошибок |
