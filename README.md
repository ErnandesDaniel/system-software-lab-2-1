# MyLang Compiler

Компилятор языка MyLang, написанный на Rust. Компилирует исходный код в исполняемый файл Windows (x86-64).

## Требования

Для работы компилятора необходимо установить следующее ПО:

### 1. Rust

Скачайте и установите с [rustup.rs](https://rustup.rs/):

```bash
rustup default stable
```

### 2. NASM (ассемблер)

**Windows:**
1. Скачайте последнюю версию компилятора с https://www.nasm.us/
2. Установите по стандартной процедуре
3. Добавьте в переменные среды PATH

**Проверка установки:**
```bash
nasm -v
```

### 3. Clang (линковщик)

**Chocolatey:**
```bash
choco install llvm
```

**Проверка установки:**
```bash
clang -v
```

## Сборка

```bash
cargo build
```

## Использование

### Сборка и запуск компилятора

```bash
cargo run -- <source_file> -o <output_dir> [options]
```

### Опции

| Опция | Описание |
|-------|-----------|
| `-o, --output <dir>` | Выходная директория (**обязательно**) |
| `-t, --target <target>` | Цель компиляции: `nasm` (по умолчанию), `llvm` |
| `--ast <file>` | Сохранить AST (диаграмма Mermaid) |
| `--cfg <file>` | Сохранить CFG (диаграмма Mermaid) |

#### Компиляция в executable

```bash
cargo run -- input.mylang -o output
```

Создаст в `output`:
- `program.asm` — ассемблер
- `program.exe` — исполняемый файл
- `assembler-code/` — папка с отдельными asm-файлами для каждой функции

**Проверка работы программы:**

Для запуска с выводом на экран используйте одну из команд:
```powershell
# Windows PowerShell
.\output\program.exe

# Или через cmd
output\program.exe

# Проверить код возврата
echo $LASTEXITCODE   # PowerShell
echo %ERRORLEVEL%    # cmd
```

**Примечание:** Для вывода на консоль (printf, puts, putchar) требуется C runtime. 
На Windows программа может запускаться без видимого вывода, если runtime не доступен.
Для полноценного вывода установите MSYS2 с MinGW-w64 (см. раздел требований).

#### Компиляция в LLVM IR

```bash
cargo run -- input.mylang -o output -t llvm
```

Создаст в `output`:
- `program.ll` — LLVM IR код (человекочитаемый)
- `program.obj` — объектный файл
- `program.exe` — исполняемый файл

**Проверка работы программы:**

Для запуска с выводом на экран используйте одну из команд:
```powershell
# Windows PowerShell
.\output\program.exe

# Или через cmd
output\program.exe

# Проверить код возврата
echo $LASTEXITCODE   # PowerShell
echo %ERRORLEVEL%    # cmd
```

**Просмотр сгенерированного LLVM IR:**
```bash
# Windows PowerShell
type output\program.ll

# Или открыть в VS Code
code output\program.ll
```

**Проверка синтаксиса LLVM IR вручную:**
```bash
# Проверить валидность LLVM IR (без компиляции)
llvm-as output\program.ll -o output\program.bc

# Дизассемблировать обратно для проверки
llvm-dis output\program.bc -o output\program_check.ll
```

**Сравнение с NASM бэкендом:**

| Фича | NASM | LLVM |
|------|------|------|
| Регистры | Вручную (rax, rcx, etc) | SSA форма (%t1, %t2) |
| Блоки | Метки с jmp | Базовые блоки с br |
| Оптимизации | Нет | Доступны через opt |
| Портативность | Только x86-64 | Любая архитектура |

#### Компиляция scheduling.mylang (корутины)

```bash
cargo run -- scheduling.mylang -o output
```

Для запуска с выводом на экран используйте одну из команд:
```powershell
# Windows PowerShell
.\output\program.exe
```

Пример с 2 параллельными корутинами, которые поочередно выполняются.

#### Компиляция с сохранением AST и CFG

```bash
cargo run -- input.mylang -o output --ast ast.mmd --cfg cfg/main.mmd
```

#### Сохранение только CFG (с отдельными файлами для каждой функции)

```bash
cargo run -- input.mylang -o output --cfg cfg/main.mmd
```

Создаст:
- `cfg/main.mmd` — CFG функции main
- `cfg/square.mmd` — CFG функции square
- `output/assembler-code/main.asm` — ассемблер для main
- `output/assembler-code/square.asm` — ассемблер для square

## Тестирование

Запуск всех тестов:
```bash
cargo test
```

Запуск только тестов компиляции и выполнения:
```bash
cargo test test_exe
```

## Этапы компиляции

1. **Лексер** → токены
2. **Парсер** → AST
3. **Семантический анализ** → проверка типов, таблица символов
4. **IR генератор** → промежуточное представление (IR)
5. **Codegen** → выбор бэкенда:
   - **NASM** → ассемблер x86-64 (default)
   - **LLVM** → LLVM IR
6. **Линковка** → исполняемый файл (Clang)

## Следующие шаги

- [x] Добавить поддержку LLVM IR
- [ ] Добавить оптимизации LLVM (opt -O2)
- [ ] Добавить поддержку WebAssembly
