# MyLang Compiler

Компилятор кастомного языка MyLang на Rust. Основная цель компиляции — NASM (x86-64), также поддерживаются JVM bytecode, LLVM IR и WebAssembly.

## Требования

- Rust: `choco install rust`
- NASM: `choco install nasm`
- Clang (линковщик): `choco install llvm`
- Java JDK 21+ (для target `jvm`): `choco install openjdk`
- PHP 8.1+ с FFI (для PHP-демок): `choco install php`
- SQLite (для верификации lab-2): `choco install sqlite`

## Линты

```powershell
# Форматирование кода
cargo fmt

# Статический анализ (clippy)
cargo clippy
```

## Сборка

```powershell
cargo build
```

## Тестирование

```powershell
# Все тесты
cargo test
```

## Использование

```powershell
cargo run -- <source_file> -o <output_dir> [options]
```

### Опции

| Опция | Описание |
|-------|-----------|
| `-o, --output <dir>` | Выходная директория (**обязательно**) |
| `-t, --target <target>` | Цель компиляции: `nasm` (по умолчанию), `llvm`, `jvm`, `wasm` |
| `--optimize` | Оптимизировать (O2) при компиляции в wasm |
| `--ast <file>` | Сохранить AST (диаграмма Mermaid) |
| `--cfg <file>` | Сохранить CFG (диаграмма Mermaid) |

### Компиляция в executable (NASM)

```powershell
cargo run -- input.mylang -o output
```

Создаст в `output/`: `main.asm`, `main.obj`, `program.exe`

**Запуск:**
```powershell
.\output\program.exe
```

### Компиляция в JVM (Java bytecode)

```powershell
cargo run -- input.mylang -o output -t jvm
```

Создаст в `output/`: `.class` файлы (по одному на функцию), `RuntimeStub.java`, `MainRunner.java`.

**Запуск:**
```powershell
java -cp output RuntimeStub
```
**Для программ, использующих SHM (server.mylang), потребуется JNA:**
```powershell
java -cp "output;output/lib/jna-5.14.0.jar" RuntimeStub
```
**Отладочный вызов отдельной функции через MainRunner:**
```powershell
java -cp output MainRunner square 7
```

## Этапы компиляции

1. **Лексер** → токены
2. **Парсер** → AST
3. **Семантический анализ** → проверка типов, таблица символов
4. **IR генератор** → промежуточное представление (IR)
5. **Codegen** → выбор бэкенда (NASM / LLVM / JVM / WASM)
6. **Линковка** → исполняемый файл (Clang) / .class (javac)

---

## Структура проекта

| Путь | Назначение |
|------|------------|
| `src/` | Исходный код компилятора на Rust |
| `src/lexer/` | Лексер |
| `src/parser/` | Парсер |
| `src/ast/` | AST-ноды |
| `src/semantics/` | Семантический анализ |
| `src/ir/`, `src/ir_generator/` | Промежуточное представление |
| `src/codegen/` | NASM, LLVM, JVM, WASM кодогенераторы |
| `src/lib/jna-5.14.0.jar` | JNA для RuntimeStub (JVM target) |
| `output/` | Результаты компиляции (генерируется) |

---

## Лабораторные работы

## Системное ПО

### lab-1: Корутины + Планировщик задач (RR + SRT)

Два файла в `labs-examples/system-programms/lab-1/`:

**`input.mylang`** — демо корутин. Две бесконечные корутины печатают `1` и `2` поочерёдно через планировщик.

```powershell
cargo run -- labs-examples/system-programms/lab-1/input.mylang -o output -t nasm
.\output\program.exe
```

**`metrics.mylang`** — симулятор планировщика (вариант 19: RR(2) + SRT). Диапазон burst 4–8, средние интервалы 6 и 3. Выводит таблицу процессов и средние turnaround/wait для каждого алгоритма.

```powershell
cargo run -- labs-examples/system-programms/lab-1/metrics.mylang -o output -t nasm
.\output\program.exe
```

### lab-2: Map-Reduce конвейер (СПО)

Реализация 7 SQL-запросов (вариант 59) в виде конвейера процедур-обработчиков, соединённых
байтовыми потоками (pipe'ами). Каждый этап обработки — отдельная корутина (кооперативная
многозадачность с passive waiting через групповое ожидание).

Данные для таблиц захардкожены в `.mylang`-файлах в виде глобальных массивов.

**Файлы:**

```
labs-examples/system-programms/lab-2/
├── csv-data/                    # Тестовые CSV-файлы (для справки)
│   ├── people.csv               # Н_ЛЮДИ — люди
│   ├── studies.csv              # Н_ОБУЧЕНИЯ — информация об обучении
│   ├── students.csv             # Н_УЧЕНИКИ — студенты
│   ├── vedomosti.csv            # Н_ВЕДОМОСТИ — оценки
│   ├── types_vedomostei.csv     # Н_ТИПЫ_ВЕДОМОСТЕЙ — типы ведомостей
│   └── group_plans.csv          # Н_ГРУППЫ_ПЛАНОВ — планы групп
├── sql/                         # Верификация через SQLite
│   ├── init.sql                 # DDL + импорт CSV
│   ├── init_abs.sql             # То же с абсолютными путями
│   ├── queries.sql              # 7 запросов (как в задании)
│   ├── run_verification.cmd     # Батник: создать БД и выполнить запросы
│   └── ucheb_test.db            # Готовая БД
├── query1.mylang                # Запрос 1: INNER JOIN с фильтрами
└── ...                          # query2.mylang — query7.mylang (по мере реализации)
```

**Компиляция и запуск:**

Все результаты компиляции идут в корневую `output/`.

```powershell
# Запрос 1: INNER JOIN Н_ТИПЫ_ВЕДОМОСТЕЙ + Н_ВЕДОМОСТИ
cargo run -- labs-examples/system-programms/lab-2/query1.mylang -o output
.\output\program.exe
```

Ожидаемый вывод:
```
=== Query 1: INNER JOIN ===
Дифзачет, 2013-06-01
Дифзачет, 2013-06-02
Дифзачет, 2013-06-07
Дифзачет, 2014-01-25
=== Done ===
```

**Верификация через SQLite:**

```powershell
choco install sqlite

# Создать БД и выполнить 7 запросов
labs-examples\system-programms\lab-2\sql\run_verification.cmd

# Или вручную:
sqlite3 labs-examples\system-programms\lab-2\sql\ucheb_test.db ^
  < labs-examples\system-programms\lab-2\sql\init_abs.sql
sqlite3 -header -column labs-examples\system-programms\lab-2\sql\ucheb_test.db ^
  < labs-examples\system-programms\lab-2\sql\queries.sql
```

Ожидаемые результаты запросов:

| # | Результат |
|---|-----------|
| 1 | 4 строки (Дифзачет с датами) |
| 2 | 1 строка (Крылов Кирилл, НЗК=OK500) |
| 3 | 6 (студентов ФКТИУ без отчества) |
| 4 | 2 плана (101: 3 группы, 104: 3 группы) |
| 5 | 2 студента (Григорьев 5.0, Зайцев 5.0) |
| 6 | 4 студента (заочные 1 курс после 2012) |
| 7 | 12 строк (6 пар с одинаковыми фамилиями) |

## Виртуальные машины

### lab-1: компиляция с target под JVM

```powershell
# Компиляция в JVM (Java bytecode)
cargo run -- labs-examples/vitrual-machines/lab-1/input.mylang -o output -t jvm
java -cp output Main
```

### lab-2: Функции первого класса + замыкания

```powershell
# Компиляция в JVM (Java bytecode)
cargo run -- labs-examples/vitrual-machines/lab-2/input.mylang -o output -t jvm
java -cp output Main
```

В одном файле 7 сценариев:

| # | Сценарий | Результат |
|---|----------|-----------|
| 1 | Локальная `double(21)` | `42` |
| 2 | Функциональный литерал `square(5)` | `25` |
| 3 | Композиция `apply_twice(double, 3)` | `12` |
| 4 | Замыкание (read-only) `read_x()` | `10` |
| 5 | Замыкание (мутация) `inc_y()` ×3 | `y = 3` |
| 6 | Счётчик через замыкание `inc_count()` ×3 | `123` |
| 7 | Комбинация всего | `2730` |

### lab-4: PHP ↔ JVM через Shared Memory

```powershell
# 1. Скомпилировать MyLang-сервер в JVM
cargo run -- labs-examples/vitrual-machines/lab-4/input.mylang -o output -t jvm

# 2. Запустить PHP CLI (автоматически стартует JVM-демон)
php labs-examples/vitrual-machines/lab-4/input.php
```

### Команды

| Команда | Описание |
|---------|----------|
| `create <key> <value>` | Создать запись |
| `get <key>` | Получить значение |
| `set <key> <value>` | Обновить |
| `delete <key>` | Удалить |
| `list` | Список ключей |
| `exit` | Остановить демон и выйти |

### Как работает

```
PHP (cli_app.php) → FFI/kernel32.dll → CreateFileMapping + MapViewOfFile
  → mylang_shm.dat (4096 байт) + Win32 Event
JVM (RuntimeStub) → main() → Main.call() → Dispatch.dispatch()
  → CRUD-функции → ответ через SHM → сигнал PHP через Event
PHP читает SHM → выводит результат
```

### Протокол SHM

```
Запрос:
  [0..3]  state    int32 LE  0=idle, 1=request, 2=done, 3=exit
  [4]     opcode   byte      0=create,1=get,2=set,3=delete,4=list,5=exit
  [5..]   key\0    string    null-terminated key
  [..]    value\0  string    null-terminated value

Ответ:
  [0..3]  state    int32 LE  2=done
  [4]     result   byte      0=ok, 1=error
  [5..]   payload\0 string   null-terminated payload
```

---
