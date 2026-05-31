# lab-4: PHP ↔ MyLang через pipe (stdin/stdout)

## Компиляция

### NASM
```powershell
cargo run -- labs-examples/vitrual-machines/lab-4/input.mylang -o output -t nasm
```

### JVM
```powershell
cargo run -- labs-examples/vitrual-machines/lab-4/input.mylang -o output -t jvm
```

## Запуск

### NASM, text
```powershell
php labs-examples/vitrual-machines/lab-4/input.php
```

### NASM, binary
```powershell
php labs-examples/vitrual-machines/lab-4/input.php --protocol binary
```

### JVM, text
```powershell
php labs-examples/vitrual-machines/lab-4/input.php --target jvm
```

### JVM, binary
```powershell
php labs-examples/vitrual-machines/lab-4/input.php --target jvm --protocol binary
```

## Команды

| Команда | Описание |
|---------|----------|
| `create <key> <value>` | Создать запись key=value |
| `get <key>` | Получить значение по ключу |
| `set <key> <value>` | Обновить значение по ключу |
| `delete <key>` | Удалить запись по ключу |
| `list` | Список всех ключей |
| `exit` | Остановить сервер |
