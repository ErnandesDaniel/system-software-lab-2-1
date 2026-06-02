# MyLang Compiler

Компилятор кастомного языка MyLang на Rust. Цели компиляции — NASM (x86-64) и JVM bytecode.

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
| `-t, --target <target>` | Цель компиляции: `nasm` (по умолчанию), `jvm` |

### Компиляция в executable (NASM)

```powershell
cargo run -- input.mylang -o output
.\output\program.exe
```

### Компиляция в JVM (Java bytecode)

```powershell
cargo run -- input.mylang -o output -t jvm
java -cp output Main
```

## Этапы компиляции

1. **Лексер** → токены
2. **Парсер** → AST
3. **Семантический анализ** → проверка типов, таблица символов
4. **IR генератор** → промежуточное представление (IR)
5. **Codegen** → выбор бэкенда (NASM / JVM)

---

## Спецификация языка MyLang

```ebnf
identifier: "[a-zA-Z_][a-zA-Z_0-9]*"; // идентификатор

str: "\"[^\"\\]*(?:\\.[^\"\\]*)*\""; // строка, окруженная двойными кавычками
char: "'[^']'"; // одиночный символ в одинарных кавычках
hex: "0[xX][0-9A-Fa-f]+"; // шестнадцатеричный литерал
bits: "0[bB][01]+"; // битовый литерал
dec: "[0-9]+"; // десятичный литерал
bool: 'true'|'false'; // булевский литерал

list<item>: (item (',' item)*)?; // список элементов, разделённых запятыми


source: sourceItem*;

typeRef: {
    |builtin: 'bool'|'byte'|'int'|'uint'|'long'|'ulong'|'char'|'string';
    |custom: identifier;
    |array: typeRef 'array' '[' dec ']'; // число - размерность
};

funcSignature: identifier '(' list<arg> ')' ('of' typeRef)? {
    arg: identifier ('of' typeRef)?;
};

sourceItem: {
    |funcDef: 'def' funcSignature statement* 'end';
};

statement: { // присваивание через '='
    |if: 'if' expr 'then' statement ('else' statement)?;
    |loop: ('while'|'until') expr statement* 'end';
    |repeat: statement ('while'|'until') expr ';';
    |break: 'break' ';';
    |expression: expr ';';
    |block: ('begin'|'{') (statement|sourceItem)* ('end'|'}');
};

expr: {
    |binary: expr binOp expr; // где binOp - символ бинарного оператора
    |unary: unOp expr; // где unOp - символ унарного оператора
    |braces: '(' expr ')';
    |call: expr '(' list<expr> ')';
    |slice: expr '[' list<range> ']' { // индексация или срез массива
    ranges: expr ('..' expr)?; // from index, to
    };
    |place: identifier;
    |literal: bool|str|char|hex|bits|dec;
};
```

### Дополнения 2-го семестра

```ebnf
typeRef: {
    |builtin: 'bool'|'byte'|'int'|'uint'|'long'|'ulong'|'char'|'string';
    |custom: identifier;
    |array: typeRef 'array' '[' dec ']'; // число - размерность
    |funcType: 'def' '(' list<typeRef> ')' ('of' typeRef)?; // тип функции
};

statement: { // присваивание через '='
    |if: 'if' '(' expr ')' '{' statement* '}' ('else' 'if' '(' expr ')' '{' statement* '}')* ('else' '{' statement* '}')?;
    |loop: ('while'|'until') '(' expr ')' '{' statement* '}';
    |repeat: '{' statement* '}' ('while'|'until') '(' expr ')' ';';
    |break: 'break' ';';
    |yield: 'yield' ';';
    |expression: expr ';';
    |block: '{' (statement|sourceItem)* '}';
    |funcDef: 'def' funcSignature '{' statement* '}'; // локальная функция
};

sourceItem: {
    |funcDef: 'def' funcSignature '{' statement* '}';
    |structDef: 'struct' identifier '{' list<field> '}' {
        field: identifier 'of' typeRef ';';
    };
    |coroutineDef: 'coroutine' funcSignature '{' statement* '}';
};

expr: {
    |binary: expr binOp expr; // где binOp - символ бинарного оператора
    |unary: unOp expr; // где unOp - символ унарного оператора
    |braces: '(' expr ')';
    |call: expr '(' list<expr> ')';
    |slice: expr '[' list<range> ']' { // индексация или срез массива
    ranges: expr ('..' expr)?; // from index, to
    };
    |place: identifier;
    |literal: bool|str|char|hex|bits|dec;
    |funcLiteral: 'def' funcSignature '{' statement* '}'; // функциональный литерал
};
```

Вместе эти изменения делают функции **first-class**: 
function type — это тип данных, func literal — выражение, возвращающее функцию, 
поэтому её можно присвоить переменной или передать как аргумент. 
Поскольку внутри функции могут быть `def` (локальные функции), 
они захватывают переменные внешней области видимости, образуя **closures**.

- **Closures** — локальная функция или func literal может захватывать переменные из 
- внешней области видимости (по ссылке, мутабельно).
- **MakeClosure / CallClosure** — новые IR-опкоды (`src/ir/types.rs`) для создания и вызова замыкания.
- **Closure env** — захваченные переменные хранятся как `int[1]`-обёртки (`[[I` на JVM, 
- стековые слоты на NASM).

## Структура проекта

| Путь | Назначение |
|------|------------|
| `src/` | Исходный код компилятора на Rust |
| `src/lexer/` | Лексер |
| `src/parser/` | Парсер |
| `src/ast/` | AST-ноды |
| `src/semantics/` | Семантический анализ |
| `src/ir/`, `src/ir_generator/` | Промежуточное представление |
| `src/codegen/nasm/` | NASM кодогенератор |
| `src/codegen/jvm/` | JVM кодогенератор |
| `labs-examples/` | Лабораторные работы (см. README в каждой lab) |
| `output/` | Результаты компиляции (генерируется) |
| `src/tests/` | Тесты (unit + integration) |

## Покрытие тестами

Тесты разделены на 2 уровня:
- **Unit** (`parser_tests`, `semantics_tests`, `codegen_tests`, `jvm_tests`, `cross_target_tests`) — проверяют отдельный этап компиляции, не запуская `.exe`.
- **Integration** (`integration_tests`) — полный цикл: source → `.exe` → запуск → проверка stdout/exit code.

### Сводка текущего покрытия (~250 active, 9 ignored)

| Категория | Что покрыто | Пробелы |
|-----------|-------------|---------|
| **Управление** | `if/else`, `while`, `break` | `repeat`, `until` (codegen), `break` вне цикла (sem) |
| **Типы** | `int`, `bool`, `string` | `byte`, `uint`, `long`, `ulong`, `char` (codegen/sem/integration) |
| **Арифметика** | `+`, `-`, `*`, `/`, `%` | `/`, `%` (NASM unit) |
| **Сравнения** | `==`, `!=`, `<`, `>`, `<=`, `>=` | `!=`, `<=`, `>=` (NASM unit) |
| **Логические** | `!` (NASM unit), `&&`, `||` (integration) | `&&`, `||` (NASM/JVM unit) |
| **Побитовые** | — | `&`, `\|`, `^`, `~` (всё) |
| **Массивы** | глобальные, pool byte array | slice `..` range (всё), локальные (NASM unit) |
| **Структуры** | парсинг, NASM codegen | запись поля (NASM unit), runtime (JVM) |
| **Функции** | вызов, multi-param, return | func literal, func type (всё) |
| **Замыкания** | простые, mutate | с параметрами, nested, как аргумент/возврат |
| **Корутины** | парсинг, basic JVM valid | runtime return value, множественные |
| **JVM codegen** | базовые типы, coroutine valid | globals, struct runtime, break, `until`, `&&`/`||`, `/`, `%`, hex/bin/char literals |
| **Семантика** | базовые проверки | return type mismatch, duplicate defs, arg type mismatch, break вне цикла, undefined field, field type mismatch |

### Полный список тестов для добавления

#### 1. Parser tests (`parser_tests.rs`)

| Группа | Что тестировать | Пример |
|--------|----------------|--------|
| `repeat` | `repeat ... while expr;` / `repeat ... until expr;` | `repeat x = x + 1; while x < 10;` |
| slice `..` | `arr[0..5]`, `arr[i..j]`, `arr[i..]`, `arr[..j]` | `x = data[2..5]` |
| func literal | `f = def(a of int) of int return a+1 end` | парсинг inline def |
| func type | `f of def(int) of int` | объявление переменной с типом функции |
| Все built-in типы | `byte`, `uint`, `long`, `ulong`, `char` | `x of byte = 10` |
| Бинарные операторы | `&`, `\|`, `^`, `<<`, `>>` | `x = a & b` |
| Литералы | hex `0xFF`, binary `0b1010`, char `'a'`, bool, string escapes | `x = 0xFF` |
| Вложенные `begin`/`end` | блоки внутри блоков | `begin x = 1; begin y = 2 end end` |
| Импорты | короткая форма, C lib, несколько импортов | `import puts; import printf` |
| Struct | вложенные struct, struct с array полями, self-reference | — |
| Array literal | `[1, 2, 3]`, `["a", "b"]` | `x = [1, 2, 3]` |
| Error recovery | неполные выражения, пропущенные `end`, лишние токены | `if x then` без `end` |

#### 2. Semantics tests (`semantics_tests.rs`)

| Группа | Что тестировать |
|--------|----------------|
| Return type mismatch | `def foo() of int return "hello"; end` → ошибка |
| Missing return | `def foo() of int x = 1 end` → ошибка |
| Void returning value | `def foo() return 42; end` → ошибка |
| `break` вне цикла | `if x then break; end` → ошибка (вне while/until/repeat) |
| `break` в nested if внутри while | `while 1 { if x then { break; } }` → OK |
| Duplicate functions | `def foo() end def foo() end` → ошибка |
| Duplicate vars | `x of int; x of string;` → ошибка |
| Undefined struct field | `a.nonexistent` → ошибка |
| Field type mismatch | присвоить `string` в `int` поле → ошибка |
| Arg type mismatch | `foo("str")` при `foo(x of int)` → ошибка |
| Arg count mismatch | `foo(1,2)` при `foo(x of int)` → ошибка |
| Yield вне coroutine | `def foo() yield; end` → ошибка |
| Uninitialized variable | `x = x + 1` без пред. объявления → ошибка |
| Closure capture validation | доступ к несхваченной переменной → ошибка |
| Array index type | `arr["hello"]` → ошибка |
| Global reassignment | `global x of int = 5; x = 10` → ошибка (если запрещено) |
| Func type compatibility | присвоить def c несовпадающей сигнатурой → ошибка |

#### 3. NASM codegen tests (`codegen_tests.rs`)

| Группа | Что проверить в ASM |
|--------|---------------------|
| `/` и `%` | `idiv` инструкции |
| `&&` и `\|\|` | short-circuit jump паттерны |
| `!=`, `<=`, `>=` | `setne`, `setle`, `setge` |
| struct field write | `mov [rax+offset]` для записи поля |
| `until` loop | jump на начало пока условие ложно |
| local array assignment | `arr[i] = val` |
| slice range | копирование диапазона |
| bitwise `&`, `\|`, `^`, `~` | `and`, `or`, `xor`, `not` |
| `repeat` statement | jump паттерн |
| func literal / closure | `MakeClosure` + `CallClosure` |
| Все типы: byte, uint, long, ulong | размер операндов (8/16/32/64 бит) |
| char literal | загрузка 1-байтовой константы |
| hex/binary literal | загрузка hex/бинарной константы |
| nested blocks | сохранение/восстановление стека |
| coroutine multi-yield | состояние между yield |
| struct with array fields | смещение nested field |

#### 4. JVM codegen tests (`jvm_tests.rs`)

| Группа | Что проверить в bytecode |
|--------|--------------------------|
| Globals | объявление, чтение (`getstatic`), запись (`putstatic`) |
| Struct runtime | поле read/write + return value |
| `break` | `goto` на выход из цикла |
| `until` | инвертированное условие |
| `repeat` | `do...while` jump |
| `&&` / `\|\|` | short-circuit (`ifne`/`ifeq` + `goto`) |
| `/` и `%` | `idiv`, `irem` |
| hex literal | загрузка `0xFF` как `bipush 255` |
| binary literal | загрузка `0b1010` как `bipush 10` |
| char literal | загрузка `'a'` как `bipush 97` |
| `byte`, `uint`, `long`, `ulong` | типы полей, cast |
| closures | `invokedynamic` + bootstrap method |
| bitwise `&`, `\|`, `^`, `~` | `iand`, `ior`, `ixor`, `iconst_m1` + `ixor` |
| func literal | генерация скрытого метода |
| slice range | копирование массива |
| coroutine return value | `areturn` + resume |
| nested blocks | frame stack |

#### 5. Cross-target tests (`cross_target_tests.rs`)

| Группа | Что проверить |
|--------|---------------|
| Все binary operators | что оба таргета компилируют один IR |
| `repeat` | IR → оба codegen |
| `break` nested | IR → оба codegen |
| globals | IR → оба codegen |
| structs | IR → оба codegen |
| closures | IR → оба codegen |
| coroutines | IR → оба codegen |
| arrays & slices | IR → оба codegen |
| all types (byte..ulong) | IR → оба codegen |

#### 6. Integration tests (`integration_tests.rs`)

| Группа | Что тестировать | Пример вход/выход |
|--------|----------------|-------------------|
| **`repeat`** | repeat-while, repeat-until | вход: `\n`, выход: `OK\n` |
| **slice `..`** | копирование массива, чтение диапазона | — |
| **all built-in types** | byte sum, uint loop, long overflow | — |
| **char ops** | char сравнение, конкатенация | — |
| **bitwise** | `&`, `\|`, `^`, `~` с известным результатом | `0xFF & 0x0F` → exit code 15 |
| **hex/bin literals** | использование в выражениях | `0xFF` → exit code 255 |
| **Closures** | с параметрами, nested, как аргумент, как return | — |
| **Coroutines** | resume, return value, multi-coroutine | — |
| **Structs** | read/write поле, вложенные struct, array field | — |
| **Nested loops** | for-like паттерны, multi-level break | — |
| **Recursion** | factorial, fibonacci (контроль глубины) | — |
| **File I/O** | fopen/fgets/fputs/fclose chain | — |
| **Memory** | malloc/free цикл, утечки | — |
| **Stdlib** | sprintf, sscanf, qsort, bsearch | — |
| **EOF patterns** | пустой stdin, частичный pipe, multiple EOF | — |
| **Daemon protocol** | create/get/set/delete/list/exit через pipe | — |
| **JVM integration** | каждый из вышеперечисленных сценариев для `-t jvm` | — |
