# MyLang Parser

Компилятор языка MyLang, написанный на Rust.

## Сборка

```bash
cargo build
```

## Использование

```bash
mylang-parser <source_file> [options]
```

### Опции

| Опция | Описание |
|-------|-----------|
| `-o, --output <dir>` | Выходная директория (**обязательно**) |
| `--ast <file>` | Сохранить AST в файл (JSON) |
| `--ir <file>` | Сохранить IR в файл (JSON) |
| `--cfg <file>` | Сохранить CFG (диаграмма Mermaid) в файл |

### Примеры

#### 1. Компиляция в executable

```bash
mylang-parser input.mylang -o output_dir
```

Создаст в `output_dir`:
- `program.asm` — ассемблер
- `program.obj` — объектный файл
- `program.exe` — исполняемый файл

#### 2. Компиляция с сохранением всех промежуточных представлений

```bash
mylang-parser input.mylang -o output_dir --ast ast.json --ir ir.json --cfg cfg.mmd
```

#### 3. Сохранение AST

```bash
mylang-parser input.mylang --ast ast.json -o output_dir
```

#### 4. Сохранение IR

```bash
mylang-parser input.mylang --ir ir.json -o output_dir
```

#### 5. Сохранение CFG диаграммы

```bash
mylang-parser input.mylang --cfg cfg.mmd -o output_dir
```

## Тестирование

```bash
cargo test
```

## Структура проекта

```
src/
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
├── codegen.rs                # Генератор ассемблера x86-64 NASM
├── semantics/                # Семантический анализ
│   ├── types.rs              # SymbolTable, SemanticType
│   └── analysis.rs           # Проверка типов
├── cfg_mermaid.rs            # Генерация CFG диаграмм Mermaid
├── mermaid/                  # Генерация AST диаграмм
└── main.rs                   # CLI
```

## Этапы компиляции

1. **Лексер** → токены
2. **Парсер** → AST
3. **Семантический анализ** → проверка типов, таблица символов
4. **IR генератор** → промежуточное представление (IR)
5. **Codegen** → ассемблер x86-64 (NASM)