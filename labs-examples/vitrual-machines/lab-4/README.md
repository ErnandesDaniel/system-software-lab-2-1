# lab-4: PHP ↔ MyLang через pipe (stdin/stdout)

Демонстрация IPC через pipe: PHP запускает скомпилированный MyLang-сервер как дочерний процесс и общается с ним через stdin/stdout.

## Компиляция и запуск

```powershell
cargo run -- labs-examples/vitrual-machines/lab-4/input.mylang -o output -t nasm
php labs-examples/vitrual-machines/lab-4/input.php
```

## Команды

| Команда | Описание |
|---------|----------|
| `create <key> <value>` | Создать запись |
| `get <key>` | Получить значение |
| `set <key> <value>` | Обновить |
| `delete <key>` | Удалить |
| `list` | Список ключей |
| `exit` | Остановить сервер и выйти |

## Протокол

Текстовый, строки разделены `\n`:

```
PHP -> MyLang: "create mykey myvalue\n"
MyLang -> PHP: "OK\n"

PHP -> MyLang: "get mykey\n"
MyLang -> PHP: "OK myvalue\n"

PHP -> MyLang: "delete mykey\n"
MyLang -> PHP: "OK\n"

PHP -> MyLang: "list\n"
MyLang -> PHP: "OK key1,key2\n"

PHP -> MyLang: "exit\n"
MyLang -> PHP: "OK\n"
```

Ошибки: `ERR message\n`

## Архитектура

```
PHP (CLI) ──proc_open──→ program.exe (MyLang-сервер)
   │                        │
   ├─ stdin ──── команда ──→│  (getchar)
   └─ stdout ←── ответ ────│  (putchar)
```

- Без JNA, без JVM, без Win32 API
- Без SHM, без файлов, без Event'ов
- Сервер — чистый MyLang, ни одного extern-хелпера не требуется (только getchar/putchar)
