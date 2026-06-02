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

### NASM
```powershell
php labs-examples/vitrual-machines/lab-4/input.php
```

### JVM
```powershell
php labs-examples/vitrual-machines/lab-4/input.php --target jvm
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

## Быстрая проверка

```powershell
# 1. Компиляция (NASM)
cargo run -- labs-examples/vitrual-machines/lab-4/input.mylang -o output -t nasm

# 2. Запуск демона и проверка команд
php labs-examples/vitrual-machines/lab-4/input.php
```

Пример сессии в демоне:

```
> create name Alice
  ok
> create age 30
  ok
> get name
  value: Alice
> list
  keys:
    - name
    - age
> set age 31
  ok
> delete name
  ok
> list
  keys:
    - age
> exit
Bye!
```
