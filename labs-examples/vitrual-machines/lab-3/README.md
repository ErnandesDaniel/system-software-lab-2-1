# lab-3: Mark-and-Sweep GC — Fibonacci Logger + OOM Stress Test

GC написан на MyLang, компилируется в оба таргета (NASM и JVM).

Три режима:
1. **GC_ON** — 50 чисел Фибоначчи, кольцевой буфер на 10, GC каждые 5 итераций → live=640 байт
2. **GC_OFF** — те же 50 чисел, GC отключён → live=3200 байт (все 50 записей)
3. **STRESS** — 2000 × malloc(1MB) с касанием страниц → OOM на NASM, успех на JVM

## Отличия от оригинального GC

| | Было | Стало |
|---|---|---|
| **Mark** | Все объекты — корни | Только handle'ы из `log_ring[10]` |
| **Sweep** | Сдвиг массива (ломает handle'ы) | free + переиспользование слота |
| **Нагрузка** | alloc 1MB в цепочку | Фибоначчи + логгирование |
| **Демо OOM** | 5000 итераций 1MB (долго, нестабильно) | 2000 × 1MB с касанием страниц |

## Компиляция и запуск

### NASM

```powershell
cargo run -- labs-examples/vitrual-machines/lab-3/input.mylang -o output -t nasm
.\output\program.exe
```

Проверить exit code после стресс-теста:
```powershell
.\output\program.exe; Write-Output "Exit: $LASTEXITCODE"
```

При недостатке RAM (менее ~4GB) NASM упадёт с `Exit: -1073741819` (0xC0000005).

### JVM

```powershell
cargo run -- labs-examples/vitrual-machines/lab-3/input.mylang -o output -t jvm
java -cp output Main
```

JVM всегда завершается успешно — встроенный GC управляет памятью.

## Ожидаемый вывод

```
========================================
  Mark-and-Sweep GC — Fibonacci Logger
  GC_ON vs GC_OFF — сравнение памяти
========================================

========================================
  MODE: GC_ON
========================================
  fib(0) = 0 ... fib(49) = ...
  [GC #1] before=320 after=320 freed=0 objects=0
  ...
  [GC #11] before=3200 after=640 freed=2560 objects=0
--- GC_ON: total_alloc=3200 total_freed=2560 live=640

========================================
  MODE: GC_OFF
========================================
  fib(0) = 0 ... fib(49) = ...
--- GC_OFF: total_alloc=3200 total_freed=0 live=3200

========================================
  STRESS TEST: malloc(1MB) без free()
========================================
  0 MB
  500 MB
  1000 MB
  1500 MB
  --- NASM: OOM / JVM: Все 2000 MB ---
```

## Сравнение таргетов

| | NASM | JVM |
|---|---|---|
| GC_ON | live=640, 50 итераций | live=640, 50 итераций |
| GC_OFF | live=3200, растёт | live=3200, растёт |
| STRESS (2000MB) | OOM при <4GB RAM | Все 2000 MB без ошибок |

Вывод: **наш GC портабелен**, но на JVM он не критичен (встроенный GC). На NASM без него — утечка и OOM.
