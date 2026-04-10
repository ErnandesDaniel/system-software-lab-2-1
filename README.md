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

### Пример программы на MyLang

```mylang
extern def getchar() of int end
extern def putchar(c of int) of int end
extern def puts(s of string) end
extern def time(dummy of int) of int end
extern def srand(seed of int) end
extern def rand() of int end

def square(x of int) of int
    return x * x;
end

def main() of int
    i = 1;
    while i < 5 {
        i = i + 1;
    }
    t = time(0);
    srand(t);
    r = rand();
    putchar(65);
    putchar(10);
    puts("Hello, World!");
    return r;
end
```

## Тестирование

Запуск всех тестов:
```bash
cargo test
```

Запуск только тестов компиляции и выполнения:
```bash
cargo test test_exe
```

## Структура проекта

```
src/
├── main.rs                   # CLI и основная логика
├── ast.rs                    # AST типы
├── lexer.rs                  # Лексер
├── parser/                   # Парсер
│   ├── mod.rs
│   ├── expressions.rs
│   ├── functions.rs
│   └── statements.rs
├── ir.rs                     # IR типы и опкоды
├── ir_generator/             # Генератор IR из AST
│   ├── mod.rs
│   ├── expressions.rs
│   └── statements.rs
├── codegen.rs                # Генератор ассемблера x86-64
├── semantics/                # Семантический анализ
│   ├── types.rs              # SymbolTable, SemanticType
│   └── analysis.rs           # Проверка типов
├── cfg_mermaid.rs            # Генерация CFG диаграмм Mermaid
├── mermaid/                  # Генерация AST диаграмм Mermaid
└── tests.rs                  # Тесты
```

## Этапы компиляции

1. **Лексер** → токены
2. **Парсер** → AST
3. **Семантический анализ** → проверка типов, таблица символов
4. **IR генератор** → промежуточное представление (IR)
5. **Codegen** → ассемблер x86-64 (NASM)
6. **Линковка** → исполняемый файл (GCC)

## Внешние функции

Компилятор поддерживает подключение внешних функций из C runtime:

```mylang
extern def getchar() of int end
extern def putchar(c of int) of int end
extern def puts(s of string) end
extern def printf(format of string, value of int) end
extern def malloc(size of int) of string end
extern def gets(s of string) end
extern def time(dummy of int) of int end
extern def srand(seed of int) end
extern def rand() of int end

def main() of int
    puts("Hello, World!");
    return 0;
end
```

## Формат генерируемого ассемблера

Генерируется ассемблер NASM для Windows x86-64:

```nasm
bits 64
default rel
section .text

global main
extern time
extern srand
extern rand
extern putchar
extern puts
extern printf

main:
    push rbp
    mov rbp, rsp
    sub rsp, 216
BB_1:
    mov rax, 1
    mov [rbp + -8], rax
    ; ... инструкции
    leave
    ret

section .data
str_0 db 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33, 0
```

## Планировщик потоков (Thread Scheduling)

Компилятор поддерживает создание потоков с алгоритмами планирования FCFS (First Come First Serve) и SPN (Shortest Process Next).

### Синтаксис

```mylang
createThread(function_name, "FCFS")   # создать поток с планировщиком FCFS
createThread(function_name, "SPN")    # создать поток с планировщиком SPN
```

### Как это работает

1. При вызове `createThread(function_name, "scheduler")` функция помечается как поток
2. Для функций-потоков после каждой инструкции автоматически вставляется вызов `yieldThread`
3. Планировщик (реализуется отдельно в runtime) переключает между потоками после каждого yield

### Пример тестовой программы

Создайте файл `scheduling.mylang`:

```mylang
extern def puts(s of string) end
extern def putchar(c of int) of int end
extern def printf(format of string, value of int) end

def print_ones() of int
    putchar(49)
    return 0
end

def print_twos() of int
    putchar(50)
    return 0
end

def main() of int
    puts("Test start")
    createThread(print_ones, "FCFS")
    createThread(print_twos, "FCFS")
    puts("Test end")
    return 0
end
```

### Компиляция и запуск

```bash
# Компиляция
cargo run scheduling.mylang -o output

# Запуск (через MSYS2 MinGW x64)
./output/program.exe
```

### Результат компиляции

После компиляции в `output/main.asm`:

```nasm
; main - обычные вызовы без yield
call puts
; createThread(print_ones, "FCFS") - вызов потока
call print_ones
call yieldThread
; createThread(print_twos, "FCFS") - вызов потока
call print_twos
call yieldThread
call puts
```

В `output/print_ones.asm` (функция-поток):

```nasm
print_ones:
    mov eax, 49
    mov [rbp + -8], eax
    call yieldThread     ; yield после каждой инструкции
    mov ecx, eax
    call putchar
    call yieldThread
    mov eax, 0
    call yieldThread
```

### Важные особенности

- Функции, объявленные как `extern def` (внешние функции), **не** разбиваются на точки с yield
- Только функции, переданные в `createThread`, получают точки прерывания после каждой инструкции
- Обычные вызовы функций без createThread выполняются как обычно без yield