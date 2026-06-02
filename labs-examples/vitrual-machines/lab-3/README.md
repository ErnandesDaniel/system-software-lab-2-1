# lab-3: Mark-and-Sweep GC — Fibonacci Logger

Реализация Mark-and-Sweep GC на MyLang с демонстрацией через Fibonacci Logger.

## Компиляция и запуск

### NASM
```powershell
cargo run -- labs-examples/vitrual-machines/lab-3/input.mylang -o output -t nasm
.\output\program.exe
```

### JVM
```powershell
cargo run -- labs-examples/vitrual-machines/lab-3/input.mylang -o output -t jvm
java -cp output Main
```

## GC алгоритм

### Mark
Обходит корневые объекты — кольцевой буфер `log_ring[0..9]` (10 хэндлов). Для живого хэндла находит слот в `gc_data[]` и ставит `gc_marked[i] = 1`.

### Sweep
Проходит по всем `gc_data[]`, если `gc_marked[i] == 0` и `gc_size[i] != 0` — вызывает `free(gc_data[i])` и ставит `gc_size[i] = 0` (слот свободен). Если `gc_marked[i] == 1` — сбрасывает `gc_marked[i] = 0` для следующего цикла.

### Alloc (`gc_alloc`)
Ищет свободный слот (`gc_size[i] == 0`) или добавляет новый, вызывает `malloc(size)`, сохраняет в `gc_data[idx]`.

## Режимы

1. **GC_ON** — 50 чисел Фибоначчи, кольцевой буфер на 10 логов, GC каждые 5 итераций → `live=640` (чистит старые логи)
2. **GC_OFF** — те же 50 чисел, GC отключён → `live=3200` (все 50 записей остаются)

## Ожидаемый вывод

```
================================================
  Mark-and-Sweep GC — Fibonacci Logger
  GC_ON vs GC_OFF — сравнение
================================================

========================================
  MODE: GC_ON
========================================
  fib(0) = 0 ... fib(49) = -811192543
  [GC #1] before=320 after=320 freed=0 objects=0
  ...
  [GC #11] before=3200 after=640 freed=2560 objects=0
--- GC_ON: total_alloc=3200 total_freed=2560 live=640 gc_runs=11

========================================
  MODE: GC_OFF
========================================
  fib(0) = 0 ... fib(49) = -811192543
--- GC_OFF: total_alloc=3200 total_freed=0 live=3200

================================================
  Вывод:
  GC_ON  → freed>0 live<alloc — GC чистит
  GC_OFF → freed=0 live=alloc — GC не чистит
================================================
```