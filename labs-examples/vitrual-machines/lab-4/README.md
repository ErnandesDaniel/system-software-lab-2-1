# lab-4: PHP ↔ JVM взаимодействие

Демонстрация межпроцессного взаимодействия (IPC) между PHP (CLI) и JVM (дочерний процесс).

## Компиляция и запуск

```powershell
cargo run -- labs-examples/vitrual-machines/lab-4/input.mylang -o output -t jvm
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
| `exit` | Остановить демон и выйти |

## Способы IPC

Доступны два протокола обмена, переключаются через флаг `--pipe`:

### --pipe=text (по умолчанию)

```
PHP ──stdin──→ JVM: create\0key\0value\0
PHP ←─stdout── JVM: 0\0payload\0
```

Команды и ответы — null-terminated строки. Можно запустить JVM-процесс вручную и общаться через echo:

```powershell
java -cp output RuntimeStub --pipe=text
```

### --pipe=binary

```
PHP ──stdin──→ JVM: [opcode:1B][key_len:2B LE][val_len:2B LE][key][value]
PHP ←─stdout── JVM: [result:1B][payload_len:2B LE][payload]
```

Бинарный протокол с фиксированными полями — без разделителей, всё по длине.

## Архитектура

Оба режима работают через `proc_open()` в PHP (дочерний JVM-процесс). Протокол переключается опцией `--pipe`:

```
PHP (CLI) ──proc_open──→ JVM (RuntimeStub)
   │                        │
   ├─ stdin ──── команда ──→│
   └─ stdout ←── ответ ────│
```

```
PHP → JVM Request:  create key value
                    get key
                    delete key
                    list
                    exit
PHP ← JVM Response: OK payload
                    ERROR description
```
