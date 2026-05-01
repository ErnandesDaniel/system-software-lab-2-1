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

### 4. PHP

PHP требуется для запуска WebAssembly тестов.

**Chocolatey:**
```bash
choco install php
```

**Проверка установки:**
```bash
php -v
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
| `-t, --target <target>` | Цель компиляции: `nasm` (по умолчанию), `llvm`, `jvm`, `wasm` |
| `--optimize` | Оптимизировать (O2) при компиляции в wasm |
| `--ast <file>` | Сохранить AST (диаграмма Mermaid) |
| `--cfg <file>` | Сохранить CFG (диаграмма Mermaid) |

**Примечание:** Для target `jvm` используйте файл `input_jvm.mylang`, так как циклы (`while`) и условия (`if`) пока не работают на JVM.

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

| Фича | NASM | LLVM | JVM | WebAssembly |
|------|------|------|-----|-------------|
| Регистры | Вручную (rax, rcx, etc) | SSA форма (%t1, %t2) | Стековая машина | SSA форма |
| Блоки | Метки с jmp | Базовые блоки с br | Метки с goto | Базовые блоки с br |
| Оптимизации | Нет | Доступны через opt | JIT JVM | Доступны через opt |
| Портативность | Только x86-64 | Любая архитектура | Любая платформа с JVM | Любая архитектура |
| Runtime | Windows .exe | Windows .exe | Java Runtime | Node.js/браузер |

#### Компиляция в JVM (Java bytecode)

**Примечание:** В текущей реализации циклы (`while`) не работают на JVM target. Используйте файл `input_jvm.mylang` вместо `input.mylang` для тестирования JVM компиляции.

**Требования:**
- Java JDK 21 или выше ([скачать](https://jdk.java.net/))

**Проверка установки:**
```bash
java -version
javac -version
```

**Компиляция:**

```bash
# Используйте input_jvm.mylang для JVM (без циклов и условий, так как они пока не работают на JVM)
cargo run -- input_jvm.mylang -o output -t jvm

# Или компилируйте input.mylang для других target'ов (nasm, llvm, wasm)
# cargo run -- input.mylang -o output -t nasm
```

Создаст в `output`:
- `Main.class` — байткод JVM (для функции main)
- `FunctionName.class` — байткод для каждой функции
- `RuntimeStub.java` — Java-реализации stdlib функций (puts, printf, rand и т.д.)

**Запуск:**

```bash
# Запустить программу
java -cp output RuntimeStub

# Проверить код возврата
echo $LASTEXITCODE   # PowerShell
echo %ERRORLEVEL%    # cmd
```

**Как это работает:**

1. Каждая функция компилируется в отдельный `.class` файл
2. `RuntimeStub.java` автоматически компилируется в `RuntimeStub.class`
3. `RuntimeStub` предоставляет реализации C-функций (puts, printf, rand и т.д.) на Java
4. Программа вызывает Java-методы через `invokestatic`
5. `RuntimeStub.main()` вызывает `Main.main()` и возвращает код завершения

**Работающий пример:**

Файл `input_jvm.mylang` — простой работающий пример для JVM:

**Компиляция и запуск:**

```bash
# Компилировать программу для JVM
cargo run -- input_jvm.mylang -o output -t jvm

# Запустить main функцию
java -cp output MainRunner main

# Запустить отдельную функцию
java -cp output MainRunner square 7
```

**Вывод:**
```
Hello from JVM!
Results:
a + b = 8
a * b = 15
square(a) = 25
```

*Примечание: Для переноса строки в `printf` используйте `\n` в форматной строке (как в C). Функция `puts` автоматически добавляет перенос.*

#### Вызов из PHP

**Запуск:**

```bash
php run_input_jvm.php
```

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


## Техническое задание: Корутины и Планировщик

### Архитектура

Реализация **кооперативной многозадачности** с автоматическим планировщиком.

#### Концепция

Пользовательская `main` — это **"фиктивная"** функция инициализации. Она регистрирует корутины, а настоящая точка входа скрыта.

```mylang
// Объявление корутины
coroutine task1() of int
    puts("Task 1");
    yield;              // <-- приостановка
    puts("Task 1 done");
    return 10;
end

// "Фиктивная" main - только для инициализации
def main() of int
    task1();            // <-- автоматически создает и регистрирует корутину
    task2();            // <-- то же самое
    
    puts("Setup done");
    return 0;           // <-- здесь автоматически запускается планировщик!
end
```

**Поток выполнения:**
1. `_real_main` (скрытая) запускается
2. Вызывается `user_main` ("фиктивная" main пользователя)
3. Внутри `user_main` вызовы корутин (`task1()`) не выполняют код, а **регистрируют** корутины в планировщике
4. После `return` в `user_main` автоматически запускается `run_scheduler()`
5. Планировщик выполняет все зарегистрированные корутины до завершения

### Синтаксис

```mylang
// Объявление корутины
coroutine имя_функции(параметры) of тип_возврата
    // локальные переменные - сохраняются между yield
    counter = 0;
    
    while counter < 10 {
        // работа...
        yield;              // приостановка, возврат управления планировщику
        counter = counter + 1;
    }
    loop_end
    
    return значение;        // сигнал завершения корутины
end

// Использование в main
def main() of int
    // Просто вызываем - это auto-spawn (создание + регистрация)
    имя_функции(аргументы);
    
    // Можно вызвать yield в самой main
    // для кооперации с корутинами
    yield;
    
    return 0;               // запуск планировщика
end
```

### Ключевые слова

| Ключевое слово | Описание |
|----------------|----------|
| `coroutine` | Объявление корутины |
| `yield` | Приостановка выполнения, возврат управления планировщику |

### Генерация кода

#### 1. Точка входа (скрытая)

```nasm
_real_main:
    call init_scheduler
    call user_main              ; "фиктивная" main
    call run_scheduler          ; запуск всех корутин
    call wait_all_tasks
    mov eax, [exit_code]
    ret
```

#### 2. Корутина (state machine)

Каждая корутина компилируется в state machine с точками входа:

```nasm
task1_coroutine:
    ; Switch по сохраненному PC (program counter)
    mov rax, [rcx + state.rip]
    
    cmp rax, 0
    je .start
    cmp rax, 1
    je .after_yield1
    ; ...
    
.start:
    call puts
    mov qword [rcx + state.rip], 1    ; save next PC
    ret                                ; yield!
    
.after_yield1:
    ; продолжение после yield
```

#### 3. Auto-spawn при вызове

При вызове корутины внутри main:

```mylang
def main() of int
    task1();        // task1 - coroutine
end
```

Генерируется:

```nasm
user_main:
    call create_coroutine_task1     ; выделить память для состояния
    call scheduler_add              ; добавить в список
    
    ; ...
    ret
```

### Структура данных корутины (MyLang)

```mylang
struct CoroutineState {
    rip of int;              // точка входа (instruction pointer)
    rsp of int;              // saved stack pointer
    rbx of int;              // saved registers
    rbp of int;
    r12 of int;
    r13 of int;
    r14 of int;
    r15 of int;
    finished of int;         // флаг завершения
    return_value of int;     // возвращенное значение
    locals of int[64];       // сохраненные локальные переменные
}

struct Scheduler {
    coroutines of int[100];  // массив handle'ов (указателей)
    count of int;
    current_index of int;
}
```

### Планировщик (MyLang)

**Алгоритм:** Round-robin (циклический)

```mylang
// Глобальные переменные планировщика
global scheduler of Scheduler;

def scheduler_has_work() of int
    return scheduler.count > 0;
end

def get_next_coroutine() of int
    if scheduler.count == 0 {
        return 0;
    }
    
    handle = scheduler.coroutines[scheduler.current_index];
    scheduler.current_index = scheduler.current_index + 1;
    
    if scheduler.current_index >= scheduler.count {
        scheduler.current_index = 0;
    }
    
    return handle;
end

def remove_coroutine(index of int) of int
    i = index;
    while i < scheduler.count - 1 {
        scheduler.coroutines[i] = scheduler.coroutines[i + 1];
        i = i + 1;
    }
    loop_end
    scheduler.count = scheduler.count - 1;
    return 0;
end

def run_scheduler() of int
    while scheduler_has_work() {
        co = get_next_coroutine();
        
        if co != 0 {
            state = cast(co, CoroutineState*);
            
            if state->finished == 0 {
                resume_coroutine(co);     // выполнить до yield
            }
            
            if state->finished {
                remove_coroutine(scheduler.current_index);
            }
        }
    }
    return 0;
end
```

### Пример использования

```mylang
coroutine worker(id of int) of int
    i = 0;
    while i < 3 {
        puts("Worker");
        i = i + 1;
        yield;
    }
    loop_end
    return id;
end

def main() of int
    worker(1);
    worker(2);
    worker(3);
    
    puts("All spawned");
    return 0;
end
```

**Вывод:**
```
All spawned
Worker
Worker
Worker
Worker
Worker
Worker
Worker
Worker
Worker
```

### Предварительные требования

Для реализации планировщика на чистом MyLang необходимо добавить в язык:

#### 1. Структуры (struct)

**Синтаксис:**
```mylang
struct CoroutineState {
    rip of int;           // instruction pointer
    rsp of int;           // stack pointer
    rbx of int;           // saved registers
    rbp of int;
    r12 of int;
    r13 of int;
    r14 of int;
    r15 of int;
    finished of int;      // flag
    return_value of int;
    locals of int[64];    // local variables storage
}

struct Scheduler {
    coroutines of int[100];   // array of pointers (handles)
    count of int;
    current_index of int;
}
```

**Использование:**
```mylang
def create_coroutine(func of int) of int
    co = alloc(sizeof(CoroutineState));
    co->rip = func;
    co->finished = 0;
    return co;
end

def is_finished(co of int) of int
    state = cast(co, CoroutineState*);
    return state->finished;
end
```

#### 2. Глобальные переменные

**Синтаксис:**
```mylang
// Глобальные переменные (вне функций)
global scheduler of Scheduler;
global current_coroutine of int;
global exit_code of int;

def init_scheduler() of int
    scheduler.count = 0;
    scheduler.current_index = 0;
    return 0;
end

def scheduler_add(handle of int) of int
    scheduler.coroutines[scheduler.count] = handle;
    scheduler.count = scheduler.count + 1;
    return 0;
end
```

**Требования:**
- Глобальные переменные должны инициализироваться нулем (или заданным значением)
- Доступны из любой функции
- Сохраняют значение между вызовами
- Линкуются в секцию `.data`

### Этапы реализации

#### Этап 0: Предварительные требования (ДО корутин)
1. **Структуры:**
   - Токены: `struct`, `->`, `.`, `sizeof`
   - AST: `StructDef`, `StructFieldAccess`, `StructPointerAccess`
   - Генерация: выравнивание полей, доступ по offset
   - Функция: `alloc(size)` - выделение памяти из кучи

2. **Глобальные переменные:**
   - Парсинг объявлений вне функций
   - Генерация в секцию `.data`
   - Доступ через label (например: `mov rax, [scheduler_count]`)

#### Этап 1: Корутины
3. **Лексер/Парсер:** добавить токены `coroutine`, `yield`
4. **AST:** добавить ноды `CoroutineDef`, `YieldStatement`, `CoroutineCall`
5. **Анализ:** определить какие функции - корутины
6. **Генерация кода:**
   - State machine для корутин
   - Скрытая `_real_main`
   - Auto-spawn при вызове корутин
7. **Runtime (ASM):**
   - `init_scheduler`
   - `scheduler_add`
   - `run_scheduler`
   - `resume_coroutine`
   - Сохранение/восстановление контекста (registers)
