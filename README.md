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
    |if: 'if' expr 'then' statement ('else' statement)?;
    |loop: ('while'|'until') expr statement* 'end';
    |repeat: statement ('while'|'until') expr ';';
    |break: 'break' ';';
    |yield: 'yield' ';';
    |expression: expr ';';
    |block: ('begin'|'{') (statement|sourceItem)* ('end'|'}');
    |funcDef: 'def' funcSignature statement* 'end'; // локальная функция
};

sourceItem: { // дополнено structDef и coroutineDef
    |funcDef: 'def' funcSignature statement* 'end';
    |structDef: 'struct' identifier '{' list<field> '}' {
        field: identifier ('of' typeRef)? ';';
    };
    |coroutineDef: 'coroutine' funcSignature statement* 'end';
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
    |funcLiteral: 'def' funcSignature statement* 'end'; // функциональный литерал
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
