# System Software Lab 2-1: MyLang Compiler + PHP↔JVM via SHM

Два компонента:
1. **MyLang Compiler** (Rust) — компилятор кастомного языка в NASM/LLVM/JVM/WASM
2. **PHP↔JVM Demo** — PHP общается с Java-демоном через shared memory (mmap'd file) по бинарному протоколу

---

## PHP↔JVM Demo

PHP (FFI → kernel32.dll) ↔ SHM-файл (mylang_shm.dat) ↔ JVM-демон (RandomAccessFile)

### Быстрый старт

Всё управление — из одного PHP-процесса:

```powershell
php cli_app.php start       # запустить JVM-демон в фоне
php cli_app.php              # интерактивный режим
```

Если демон ещё не запущен, можно запустить его внутри CLI командой `start`.

### Команды CLI

| Команда | Пример | Описание |
|---------|--------|----------|
| `start` | `start` | Запустить JVM-демон (если не запущен) |
| `create` | `create note1 Hello` | Создать запись по ключу |
| `get` | `get note1` | Получить значение |
| `set` | `set note1 World` | Обновить значение |
| `delete` | `delete note1` | Удалить по ключу |
| `list` | `list` | Список ключей |
| `exec` | `exec square 7` / `exec add 3 5` | Выполнить builtin-функцию |
| `exit` | `exit` | Остановить демон и выйти |

### Builtin функции (exec)

| Функция | Аргументы | Пример |
|---------|-----------|--------|
| `square` | 1 int | `exec square 7` → 49 |
| `add` | 2+ ints | `exec add 3 5 2` → 10 |

### SHM Бинарный протокол

Размер SHM: 4096 байт (mylang_shm.dat, mmap через CreateFileMapping).

```
Запрос:
  [0..3]  state    int32 LE  0=idle, 1=request, 2=done, 3=exit
  [4]     opcode   byte      0=create,1=get,2=set,3=delete,4=list,5=exec,6=exit
  [5..]   key\0    string    null-terminated key (или имя функции для exec)
  [..]    value\0  string    null-terminated value (или args для exec)

Ответ:
  [0..3]  state    int32 LE  2=done
  [4]     result   byte      0=ok, 1=error
  [5..]   payload\0 string   null-terminated payload (пусто если ok без данных)
```

### Структура проекта

| Файл | Назначение |
|------|------------|
| `output/MainServer.java` | JVM-демон: CRUD (HashMap) + exec (square/add) |
| `shm_client.php` | PHP FFI класс (CreateFileA → CreateFileMappingA → MapViewOfFile) |
| `cli_app.php` | PHP CLI с интерактивными командами |
| `test_shm.php` | Интеграционный тест |
| `run_daemon.bat` | (альтернатива) Запуск: `java -cp output MainServer` |
| `stop_daemon.bat` | (альтернатива) Остановка: убить java процесс |

### Примечания

- `php cli_app.php < file.txt` не работает на Windows — `fgets(STDIN)` блокируется при пайпе. Интерактивный режим работает нормально.
- Убедись, что Java JDK (21+) доступна через `java -version`, иначе `start` не сработает.

---

## MyLang Compiler

Компилятор языка MyLang, написанный на Rust.

### Требования

- Rust: `rustup default stable`
- NASM: https://www.nasm.us/ (для target nasm)
- Clang/LLVM: `choco install llvm` (для линковки и target llvm)
- Java JDK 21+: `choco install openjdk` (для target jvm)
- PHP: `choco install php` (для WASM тестов)

### Сборка

```bash
cargo build
```

### Использование

```bash
cargo run -- <source.mylang> -o <output_dir> [options]
```

| Опция | Описание |
|-------|----------|
| `-o, --output <dir>` | Выходная директория (**обязательно**) |
| `-t, --target <target>` | `nasm` (default), `llvm`, `jvm`, `wasm` |
| `--optimize` | Оптимизация O2 для wasm |
| `--ast <file>` | Сохранить AST (Mermaid) |
| `--cfg <file>` | Сохранить CFG (Mermaid) |

### Примеры

```bash
# NASM executable
cargo run -- input.mylang -o output

# JVM bytecode
cargo run -- input_jvm.mylang -o output -t jvm

# LLVM IR
cargo run -- input.mylang -o output -t llvm

# WebAssembly
cargo run -- input.mylang -o output -t wasm
```

### Тестирование

```bash
cargo test
```

### Архитектура компилятора

1. **Лексер** → токены
2. **Парсер** → AST
3. **Семантический анализ** → проверка типов, таблица символов
4. **IR генератор** → промежуточное представление
5. **Codegen** → NASM / LLVM IR / JVM bytecode / WASM
6. **Линковка** → исполняемый файл
