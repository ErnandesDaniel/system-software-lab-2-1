# MyLang Compiler

Компилятор кастомного языка MyLang на Rust. Основная цель компиляции — NASM (x86-64), также поддерживаются JVM bytecode, LLVM IR и WebAssembly.

## Требования

- Rust: `rustup default stable`
- NASM: `choco install nasm`
- Clang (линковщик): `choco install llvm`
- Java JDK 21+ (для target `jvm`): `choco install openjdk`
- PHP 8.1+ с FFI (для PHP-демок)

## Сборка

```powershell
cargo build
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

Создаст в `output/`: `main.asm`, `program.exe`

```powershell
.\output\program.exe
```

### Компиляция в JVM (Java bytecode)

```powershell
cargo run -- input_jvm.mylang -o output -t jvm
```

Создаст в `output/`: `.class` файлы, `RuntimeStub.java`, `MainRunner.java`

```powershell
java -cp output RuntimeStub
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
| `examples/` | Примеры программ на MyLang |
| `output/` | Результаты компиляции (генерируется) |

---

## Техническое задание: Корутины и Планировщик

Реализация **кооперативной многозадачности** с автоматическим планировщиком.

### Концепция

Пользовательская `main` — **"фиктивная"** функция инициализации. Она регистрирует корутины, а настоящая точка входа скрыта.

```mylang
coroutine task1() of int
    puts("Task 1");
    yield;
    puts("Task 1 done");
    return 10;
end

def main() of int
    task1();
    task2();
    puts("Setup done");
    return 0;           // здесь автоматически запускается планировщик
end
```

**Поток выполнения:**
1. `_real_main` (скрытая) запускается
2. Вызывается `user_main` ("фиктивная" main пользователя)
3. Вызовы корутин внутри `user_main` **регистрируют** их в планировщике (не выполняют код)
4. После `return` из `user_main` автоматически запускается `run_scheduler()`
5. Планировщик выполняет все зарегистрированные корутины до завершения

### Синтаксис

| Ключевое слово | Описание |
|----------------|----------|
| `coroutine` | Объявление корутины |
| `yield` | Приостановка выполнения, возврат управления планировщику |

### Генерация кода

#### Точка входа (скрытая)

```nasm
_real_main:
    call init_scheduler
    call user_main
    call run_scheduler
    call wait_all_tasks
    mov eax, [exit_code]
    ret
```

#### Корутина (state machine)

```nasm
task1_coroutine:
    mov rax, [rcx + state.rip]

    cmp rax, 0
    je .start
    cmp rax, 1
    je .after_yield1

.start:
    call puts
    mov qword [rcx + state.rip], 1
    ret

.after_yield1:
    ; продолжение после yield
```

#### Auto-spawn при вызове

```nasm
user_main:
    call create_coroutine_task1
    call scheduler_add
    ret
```

### Предварительные требования

Для реализации планировщика необходимо добавить в язык:

#### 1. Структуры (struct)

```mylang
struct CoroutineState {
    rip of int;
    rsp of int;
    rbx, rbp, r12, r13, r14, r15 of int;
    finished of int;
    return_value of int;
    locals of int[64];
}

struct Scheduler {
    coroutines of int[100];
    count of int;
    current_index of int;
}
```

Требуется: токены `struct`, `->`, `.`, `sizeof`; функция `alloc(size)`.

#### 2. Глобальные переменные

```mylang
global scheduler of Scheduler;
global exit_code of int;
```

Требуется: парсинг объявлений вне функций, генерация в секцию `.data`.

### Этапы реализации

#### Этап 0: Предварительные требования
1. **Структуры:** токены `struct`/`->`/`.`/`sizeof`, AST-ноды, кодогенерация полей по offset, `alloc(size)`
2. **Глобальные переменные:** парсинг вне функций, секция `.data`, доступ через label

#### Этап 1: Корутины
3. **Лексер/Парсер:** токены `coroutine`, `yield`
4. **AST:** ноды `CoroutineDef`, `YieldStatement`, `CoroutineCall`
5. **Семантика:** определение корутин vs обычных функций
6. **Кодогенерация:** state machine, скрытая `_real_main`, auto-spawn
7. **Runtime (ASM):** `init_scheduler`, `scheduler_add`, `run_scheduler`, `resume_coroutine`

---

## Демо: PHP↔JVM CLI (Shared Memory)

Одна из демонстрационных программ. PHP общается с JVM-демоном через разделяемую память (Win32 File Mapping), позволяя интерактивно вызывать CRUD-операции, скомпилированные из `server.mylang`.

```powershell
cargo run -- cli_app/server.mylang -o output -t jvm
php cli_app/cli_app.php
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

### Файлы демо

| Файл | Назначение |
|------|------------|
| `cli_app/server.mylang` | MyLang CRUD-сервер |
| `cli_app/cli_app.php` | Интерактивная консоль |
| `cli_app/shm_client.php` | PHP FFI: kernel32 → SHM |
| `cli_app/test_shm.php` | Интеграционный тест |
| `output/RuntimeStub.java` | Генерируется: I/O, SHM через JNA, HashMap |
| `output/MainRunner.java` | Генерируется: отладочный запускатор через рефлексию |

---
