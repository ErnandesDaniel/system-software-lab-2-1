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
| `-t, --target <target>` | Цель компиляции: `nasm` (по умолчанию), `llvm`, `wasm` |
| `--optimize` | Оптимизировать (O2) при компиляции в wasm |
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

| Фича | NASM | LLVM | WebAssembly |
|------|------|------|-------------|
| Регистры | Вручную (rax, rcx, etc) | SSA форма (%t1, %t2) | SSA форма |
| Блоки | Метки с jmp | Базовые блоки с br | Базовые блоки с br |
| Оптимизации | Нет | Доступны через opt | Доступны через opt |
| Портативность | Только x86-64 | Любая архитектура | Любая архитектура |
| Runtime | Windows .exe | Windows .exe | Node.js/браузер |

#### Компиляция в WebAssembly (для Node.js)

```bash
cargo run -- input.mylang -o output -t wasm
```

Создаст в `output`:
- `program.ll` — LLVM IR код
- `program.wasm` — WebAssembly модуль

**Вызов из Node.js:**

Создайте файл `test_wasm.js`:

```javascript
const fs = require('fs');
const path = require('path');

const wasmPath = path.join(__dirname, 'output', 'program.wasm');
const wasmBuffer = fs.readFileSync(wasmPath);

// Создаем память (минимум 1 страница = 64KB)
const memory = new WebAssembly.Memory({ initial: 1 });

// Определяем функции stdlib, которые требуются скомпилированной программе
// Эти функции будут вызываться из WASM модуля
const imports = {
    env: {
        memory: memory,
        __stack_pointer: 65600,
        
        // putchar: вывод символа (основной вывод)
        putchar: (c) => process.stdout.write(String.fromCharCode(c)),
        
        // getchar: чтение символа (-1 = EOF)
        getchar: () => -1,
        
        // puts: вывод строки
        puts: () => 0,
        
        // printf: поддержка форматирования (упрощенная)
        printf: () => 0,
        
        // malloc: аллокатор памяти
        malloc: () => 0,
        
        // free: освобождение памяти
        free: () => 0,
        
        // rand: случайное число
        rand: () => Math.floor(Math.random() * 2147483647),
        
        // srand: инициализация генератора случайных чисел
        srand: () => {},
        
        // Sleep: задержка в миллисекундах
        Sleep: () => {},
        
        // time: текущее время
        time: () => 0,
    }
};

const wasmModule = new WebAssembly.Module(wasmBuffer);
const instance = new WebAssembly.Instance(wasmModule, imports);

// Доступные экспорты:
// - square(x): функция умножения
// - main(): главная функция программы

// Пример вызова функции square
if (instance.exports.square) {
    console.log("square(5) =", instance.exports.square(5));
}

// Запуск main
instance.exports.main();

console.log("\nПрограмма завершена!");
```

**Запуск:**

```bash
node test_wasm.js
```

Пример вывода:

```
square(5) = 25
A
Program completed!
```

**Примечания:**
- Все функции stdlib (`putchar`, `getchar`, `puts`, `printf`, `malloc`, `free`, `rand`, `srand`, `Sleep`, `time`) должны быть предоставлены JavaScript-хостом через импорты
- Флаг `-Wl,--export-all` экспортирует все функции (включая внутренние типа `square`)
- Для production используйте оптимизацию для уменьшения размера

**Оптимизация размера:**

```bash
# Оптимизация размера
clang --target=wasm32 -nostdlib -Wl,--no-entry -Wl,--export-all -Wl,--strip-all -Oz -o output/program.wasm output/program.ll
```

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
- [x] Добавить поддержку WebAssembly
- [ ] Добавить оптимизации LLVM (opt -O2)
- [ ] Исправить типы возврата (void vs i32) для extern функций
- [ ] Добавить поддержку браузера (Web), не только Node.js

## WebAssembly (Node.js)

### Компиляция

```bash
# Компилирует testing_wasm.mylang в WebAssembly
cargo run -- testing_wasm.mylang -o output -t wasm
```

Создаст:
- `output/program.ll` — LLVM IR
- `output/program.wasm` — WebAssembly модуль

### Тестирование

Файл `testing_wasm.js` — консольная утилита для вызова функций из WASM модуля.

```bash
# Показать справку
node testing_wasm.js

# Доступные функции:
node testing_wasm.js factorial 5    # факториал: 120
node testing_wasm.js power 2 10   # 2^10: 1024
node testing_wasm.js sum 10 20    # сложение: 30
node testing_wasm.js diff 100 37   # вычитание: 63
node testing_wasm.js product 6 7   # умножение: 42
```

**Примеры:**
```bash
# Факториал
$ node testing_wasm.js factorial 6
720

# Степень
$ node testing_wasm.js power 3 4
81

# Сложение
$ node testing_wasm.js sum 50 50
100
```

### Как это работает

1. `testing_wasm.mylang` содержит математические функции
2. Компилятор генерирует LLVM IR, затем Clang компилирует в WASM
3. `testing_wasm.js` загружает модуль и вызывает нужную функцию по имени
