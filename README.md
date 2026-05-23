# System Software Lab 2-1: MyLang Compiler + PHP↔JVM via SHM

Три компонента:
1. **MyLang Compiler** (Rust) — компилирует `cli_app/server.mylang` в JVM `.class` файлы
2. **RuntimeStub** (Java) — рантайм с shared memory, реализациями `shm_*`/`map_*`, вызывает скомпилированный MyLang-код
3. **PHP CLI** (`cli_app.php` + `shm_client.php`) — интерактивная консоль, отправляет запросы JVM-демону через разделяемую память

---

## Полный цикл: компиляция → запуск → тестирование

### 1. Требования

- Rust: `rustup default stable`
- Java JDK 21+: `choco install openjdk`
- PHP 8.1+ с FFI: `choco install php`

### 2. Сборка компилятора

```powershell
cargo build
```

### 3. Компиляция MyLang в JVM

```powershell
cargo run -- cli_app/server.mylang -o output -t jvm
```

Результат в `output/`:
- `Main.class`, `Dispatch.class`, `Handle_create.class`, `Handle_get.class`, `Handle_set.class`, `Handle_delete.class` — скомпилированный mylang-код
- `RuntimeStub.java`, `MainRunner.java` — сгенерированные Java-обёртки

### 4. Запуск PHP CLI

PHP-приложение само запускает JVM-демон при подключении:

```powershell
php cli_app/cli_app.php
```

### 6. Основной сценарий (тестирование)

```
=== PHP FFI → JVM Daemon ===

> create user1 Alice
  ok
> create user2 Bob
  ok
> get user1
  value: Alice
> set user1 Charlie
  ok
> get user1
  value: Charlie
> list
  keys:
    - user1
    - user2
> delete user2
  ok
> list
  keys:
    - user1
> exit
Bye!
```

### Как это работает

```
PHP (cli_app.php)
  ↓ FFI → kernel32.dll
  ↓ CreateFileMapping + MapViewOfFile → mylang_shm.dat (4096 байт)
  ↓ Win32 Event — сигнал JVM-демону
JVM (RuntimeStub)
  ↓ main() → инициализация SHM
  ↓ Main.call() → запуск mylang-кода
  ↓ Dispatch.dispatch() → handle_create/handle_get/handle_set/handle_delete
  ↓ Ответ через SHM → сигнал PHP через Event
PHP читает SHM → выводит результат
```

### Протокол SHM

```
Запрос:
  [0..3]  state    int32 LE  0=idle, 1=request, 2=done, 3=exit
  [4]     opcode   byte      0=create,1=get,2=set,3=delete,4=list,5=exit
  [5..]   key\0    string    null-terminated key
  [..]    value\0  string    null-terminated value

Ответ:
  [0..3]  state    int32 LE  2=done
  [4]     result   byte      0=ok, 1=error
  [5..]   payload\0 string   null-terminated payload
```

---

## Структура проекта

| Файл | Назначение |
|------|------------|
| `cli_app/server.mylang` | Исходный код на MyLang (обработчики create/get/set/delete + main-цикл) |
| `src/` | Компилятор MyLang на Rust |
| `output/Main.class` | JVM-байткод: main() из server.mylang |
| `output/Dispatch.class` | JVM-байткод: dispatch-таблица по opcode |
| `output/Handle_*.class` | JVM-байткод: CRUD-функции |
| `output/RuntimeStub.java` | Java-рантайм (shared memory, вызовы mylang-кода) |
| `output/MainRunner.java` | Java-загрузчик для вызова функций по имени |
| `cli_app/cli_app.php` | PHP CLI с интерактивными командами |
| `cli_app/shm_client.php` | PHP FFI класс (kernel32 → CreateFileMapping, MapViewOfFile, Event) |
| `cli_app/test_shm.php` | Интеграционный тест |

## Команды PHP CLI

| Команда | Пример | Описание |
|---------|--------|----------|
| `create` | `create note1 Hello` | Создать запись по ключу |
| `get` | `get note1` | Получить значение |
| `set` | `set note1 World` | Обновить значение |
| `delete` | `delete note1` | Удалить по ключу |
| `list` | `list` | Список ключей |
| `exit` | `exit` | Остановить JVM-демон и выйти |

## Примечания

- `php cli_app/cli_app.php < file.txt` не работает на Windows — `fgets(STDIN)` блокируется при пайпе. Только интерактивный режим.
- Если `mylang_shm.dat` остался после аварийного завершения, удалите вручную перед перезапуском.
- Компилятор также поддерживает цели `nasm`, `llvm`, `wasm` (см. `cargo run -- --help`).
