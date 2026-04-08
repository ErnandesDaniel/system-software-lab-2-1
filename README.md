# MyLang Compiler

Компилятор языка MyLang, написанный на Rust. Компилирует исходный код в исполняемый файл Windows (x86-64).

## Установка

### Требования

- **Rust** — [установить](https://rustup.rs/)
- **NASM** — ассемблер для x86-64
- **GoLink** — линковщик для Windows

### Windows

1. Скачайте NASM с https://www.nasm.us/ и установите
2. Скачайте GoLink с https://www.godevtool.com/ и расположите в той же папке, что и NASM
3. Добавьте папку с NASM и GoLink в PATH

Проверка установки:
```bash
nasm -v
GoLink.exe /?
```

## Сборка

```bash
cargo build --release
```

## Использование

```bash
mylang-parser <source_file> -o <output_dir> [options]
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
mylang-parser input.mylang -o output
```

Создаст в `output`:
- `program.asm` — ассемблер
- `program.exe` — исполняемый файл

#### Компиляция с сохранением AST и CFG

```bash
mylang-parser input.mylang -o output --ast ast.mmd --cfg cfg.mmd
```

#### Сохранение только AST

```bash
mylang-parser input.mylang --ast ast.mmd -o output
```

#### Сохранение только CFG

```bash
mylang-parser input.mylang --cfg cfg.mmd -o output
```

### Пример программы на MyLang

```mylang
def main() of int
    i = 1;
    while i < 5 {
        i = i + 1;
    }
    return i;
end
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
├── codegen.rs                # Генератор ассемблера x86-64 (NASM)
├── semantics/                # Семантический анализ
│   ├── types.rs              # SymbolTable, SemanticType
│   └── analysis.rs           # Проверка типов
├── cfg_mermaid.rs            # Генерация CFG диаграмм Mermaid
├── mermaid/                  # Генерация AST диаграмм Mermaid
└── main.rs                   # CLI
```

## Этапы компиляции

1. **Лексер** → токены
2. **Парсер** → AST
3. **Семантический анализ** → проверка типов, таблица символов
4. **IR генератор** → промежуточное представление (IR)
5. **Codegen** → ассемблер x86-64 (NASM)
6. **Линковка** → исполняемый файл (GoLink)

## Внешние функции

Компилятор поддерживает подключение внешних функций из C runtime:

```mylang
extern def getchar() of int end
extern def putchar(c of int) end
extern def puts(s of string) end
extern def printf(format of string, value of int) end

def main() of int
    puts("Hello, World!");
    return 0;
end
```