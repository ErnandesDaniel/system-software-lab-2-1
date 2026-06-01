# lab-3: Mark-and-Sweep Garbage Collector

## Что это

Реализация алгоритма «Mark and Sweep» как **обычная MyLang-программа**, а не встроенный рантайм.
GC написан на MyLang и компилируется в оба таргета (NASM и JVM).

## Структура GC

### Хранение объектов

Четыре параллельных глобальных массива, каждый на 2048 слотов:

```
gc_marked[2048] of int    // флаг пометки (0/1)
gc_refs[2048] of int      // ссылка на другой объект (1-based, 0 = нет ссылки)
gc_size[2048] of int      // размер аллоцированной памяти
gc_data[2048] of string   // указатель на память (malloc)
gc_count of int            // сколько слотов занято
```

### Функции

- **`gc_alloc(size, ref)`** — аллоцирует `size` байт через `bounded_malloc`, обнуляет память,
  регистрирует объект в таблице, возвращает 1-based индекс. Если достигнут лимит `heap_limit`
  или упал `malloc` — возвращает `-1`.
- **`mark(idx)`** — рекурсивно обходит граф объектов по `gc_refs`, помечая достижимые.
  Если `gc_refs[idx] != 0`, вызывает `mark(gc_refs[idx] - 1)`.
- **`sweep()`** — проходит все объекты; непомеченные (`gc_marked[i] == 0`) освобождает
  через `bounded_free`, помеченные сбрасывает (`gc_marked[i] = 0`).
  После удаления уплотняет массивы (сдвиг влево).
- **`gc_run()`** — вызывает `mark` для всех слотов, затем `sweep`.

### Ограничение памяти

```
global heap_used of int = 0
global heap_limit of int = 100000    // 100 KB
```

`bounded_malloc` отслеживает `heap_used`. `gc_alloc` проверяет `heap_used + size > heap_limit`
**до** вызова `bounded_malloc` — если лимит превышен, сразу возвращает `-1` (OOM).
`bounded_free` уменьшает `heap_used` при освобождении.

## Сценарий тестирования

Программа последовательно выполняет два режима, разделённые флагом `gc_enabled`:

### GC_ON (первые 30 итераций)

```
prev = 0
i = 0
while i < 30 {
    p = gc_alloc(1024, prev)    // новый объект, ссылается на предыдущий
    prev = p                     // цепочка: 1 → 2 → 3 → … → N
    if i % 10 == 0 then gc_run()
    i = i + 1
}
```

Каждый объект хранит ссылку на предыдущий (`gc_refs[idx] = ref`).
Получается односвязный список: первый объект ссылается на 0 (корень),
второй на первый, третий на второй и т.д.

**Почему `freed_objects = 0`:** все объекты в цепочке достижимы
из последнего. `mark()` проходит по `gc_refs` от последнего к первому —
ни один объект не оказывается непомеченным. Это корректное поведение:
GC не удаляет достижимые объекты. Чтобы GC кого-то удалил, нужно потерять
ссылку (например, не обновлять `prev` на каждом шаге).

### GC_OFF (до OOM, до 200 итераций)

Тот же цикл, но `gc_enabled = 0`. GC не запускается. Память растёт,
пока `heap_used + 1024 > heap_limit` не сработает в `gc_alloc`.

Heap quota = 100 000 байт, одна аллокация = 1024 байта.
OOM наступает примерно на 97-й итерации (97 × 1024 = 99 328 ≤ 100 000,
98 × 1024 > 100 000 — последняя аллокация не влезает).

### JVM

На JVM оба режима проходят 200 итераций без OOM.
Даже при `gc_enabled = 0` JVM имеет собственный встроенный GC,
поэтому `heap_limit` никогда не достигается:
освобождением памяти управляет рантайм, а не MyLang-код.

## Сравнение таргетов

| | NASM | JVM |
|---|---|---|
| GC_ON | Проходит 30 итераций, ~30 KB | Проходит 200 итераций |
| GC_OFF | OOM на ~97 итерации | Проходит 200 итераций |
| Причина | Свой GC написан на MyLang, quota честно работает | JVM GC встроен в рантайм, quota не срабатывает |

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
=== Mark-and-Sweep GC Demo ===
GC_ON vs GC_OFF comparison

=== MODE: GC_ON ===
  iter 5: 6 objects, 6144 bytes allocated
  iter 10: 11 objects, 11264 bytes allocated
  [GC #1] before=11264 after=11264 freed_objects=0
  iter 15: 16 objects, 16384 bytes allocated
  iter 20: 21 objects, 21504 bytes allocated
  [GC #2] before=21504 after=21504 freed_objects=0
  iter 25: 26 objects, 26624 bytes allocated
  [GC #3] before=30720 after=30720 freed_objects=0
--- GC_ON done: 30720 bytes ---

=== MODE: GC_OFF ===
  iter 0: 1 KB allocated
  iter 20: 21 KB allocated
  iter 40: 41 KB allocated
  iter 60: 61 KB allocated
  iter 80: 81 KB allocated
OOM on iter 97! allocated 97 KB
```

### JVM

```
=== MODE: GC_ON ===
  (аналогично NASM, но может пройти все 200 итераций)

=== MODE: GC_OFF ===
  iter 0: 1 KB allocated
  iter 20: 21 KB allocated
  …
--- Allocated 200 KB without OOM ---
```

## Исходный код

- `input.mylang` — полный код GC + тестовый сценарий
- Все импорты (`malloc`, `free`, `printf`, `puts`, `putchar`) — библиотечные функции,
  подключённые через `import`
