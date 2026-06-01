# lab-3: Mark-and-Sweep Garbage Collector

## Что это

Реализация алгоритма «Mark and Sweep» для NASM-таргета — как **обычная MyLang-программа**, а не встроенный рантайм.

GC состоит из трёх функций:

- `mark(idx)` — рекурсивно обходит граф объектов (по `gc_refs`), помечая достижимые
- `sweep()` — проходит все объекты, освобождает непомеченные, уплотняет массив
- `gc_run()` — вызывает `mark` для всех объектов, затем `sweep`

Учёт аллокаций идёт через `bounded_malloc`/`bounded_free` — врапперы над системным `malloc`/`free` c лимитом `heap_limit` (100 KB).

## Два режима

| Режим | GC | Результат |
|-------|----|-----------|
| `gc_enabled = 1` | Работает каждые 10 итераций | Память стабильна, программа завершается без OOM |
| `gc_enabled = 0` | Отключён | Память растёт, `gc_alloc` возвращает `-1` при превышении `heap_limit` (OOM) |

## Компиляция и запуск

### NASM target

```powershell
cargo run -- labs-examples/vitrual-machines/lab-3/input.mylang -o output -t nasm
.\output\program.exe
```

### JVM target

```powershell
cargo run -- labs-examples/vitrual-machines/lab-3/input.mylang -o output -t jvm
java -cp output Main
```

## Ожидаемый вывод

### NASM

```
=== MODE: GC_ON ===
  iter 5: 6 objects, 6144 bytes allocated
  [GC #N] before=… after=… freed_objects=0
--- GC_ON done: ~30 KB ---

=== MODE: GC_OFF ===
  iter 0: 1 KB allocated
  …
OOM on iter ~97! allocated ~97 KB
```

### JVM

```
=== MODE: GC_ON ===
  (аналогично NASM)

=== MODE: GC_OFF ===
  iter 0: 1 KB allocated
  …
--- Allocated 200 KB without OOM ---
```

## Анализ

- На **NASM**: GC_OFF упирается в `heap_limit = 100000` (~97 итераций по 1024 байт). GC_ON работает стабильно, GC освобождает недостижимые объекты.
- На **JVM**: оба режима проходят 200 итераций без OOM. Даже при `gc_enabled = 0` JVM собственный GC продолжает работать, поэтому `heap_limit` не достигается (хотя формально это неспровержимо без выключения JVM GC).
