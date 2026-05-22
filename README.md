# System Software Lab 2-1: MyLang Compiler + PHP↔JVM via SHM

Два компонента:
1. **MyLang Compiler** (Rust) — компилятор кастомного языка в NASM/LLVM/JVM/WASM
2. **PHP↔JVM Demo** — PHP общается с Java-демоном через shared memory (mmap'd file) по бинарному протоколу

---

## PHP↔JVM Demo

PHP (FFI → kernel32.dll) ↔ SHM-файл (mylang_shm.dat) ↔ JVM-демон (RandomAccessFile)

### Быстрый старт

```powershell
php cli_app.php
```

Демон запускается автоматически. После `exit` демон останавливается и CLI завершается.

### Команды CLI

| Команда | Пример | Описание |
|---------|--------|----------|
| `start` | `start` | Запустить JVM-демон внутри сессии |
| `create` | `create note1 Hello` | Создать запись по ключу |
| `get` | `get note1` | Получить значение |
| `set` | `set note1 World` | Обновить значение |
| `delete` | `delete note1` | Удалить по ключу |
| `list` | `list` | Список ключей |
### SHM Бинарный протокол

Размер SHM: 4096 байт (mylang_shm.dat, mmap через CreateFileMapping).

```
Запрос:
  [0..3]  state    int32 LE  0=idle, 1=request, 2=done, 3=exit
  [4]     opcode   byte      0=create,1=get,2=set,3=delete,4=list,5=exit
  [5..]   key\0    string    null-terminated key
  [..]    value\0  string    null-terminated value

Ответ:
  [0..3]  state    int32 LE  2=done
  [4]     result   byte      0=ok, 1=error
  [5..]   payload\0 string   null-terminated payload (пусто если ok без данных)
```

### Структура проекта

| Файл | Назначение |
|------|------------|
| `output/MainServer.java` | JVM-демон: CRUD (HashMap) |
| `shm_client.php` | PHP FFI класс (CreateFileA → CreateFileMappingA → MapViewOfFile) |
| `cli_app.php` | PHP CLI с интерактивными командами |
| `test_shm.php` | Интеграционный тест |

### Примечания

- `php cli_app.php < file.txt` не работает на Windows — `fgets(STDIN)` блокируется при пайпе. Интерактивный режим работает нормально.
- Для запуска требуется Java JDK 21+ (`java -version`).

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
