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

### 3. GCC (линковщик)

**Windows (через MSYS2):**

1. Скачайте установщик MSYS2 с https://www.msys2.org/
2. Установите и запустите **MSYS2 MSYS** (из меню Пуск)
3. Выполните:
   ```bash
   pacman -Syu
   ```
4. Если система попросит перезапустить терминал — закройте и откройте снова, затем повторите:
   ```bash
   pacman -Syu
   ```
5. Откройте **MSYS2 MinGW x64** (из меню Пуск)
6. Установите MinGW-w64 GCC:
   ```bash
   pacman -S mingw-w64-x86_64-gcc
   ```
7. Добавьте в PATH (переменные среды Windows):
   ```
   C:\msys64\mingw64\bin
   ```

**Проверка установки:**
```bash
gcc -v
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
| `--ast <file>` | Сохранить AST (диаграмма Mermaid) |
| `--cfg <file>` | Сохранить CFG (диаграмма Mermaid) |

### Примеры

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

**Важно:** putchar/puts могут не выводить на экран (зависит от MSYS2/GCC линковки).

Создаст в `output`:
- `program.exe` — исполняемый файл с корутинами
- Отдельные `.asm` файлы для каждой функции

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
5. **Codegen** → ассемблер x86-64 (NASM)
6. **Линковка** → исполняемый файл (GCC)