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
