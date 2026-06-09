# Lab 3 — FTP-like CLI для Ext3

## Задание

Реализовать программу для инспектирования образа файловой системы Ext3 в диалоговом режиме с
интерфейсом, аналогичным протоколу PASSIVE FTP.

**Общий алгоритм работы:**
1. Проверить, поддерживается ли файловая система на заданном образе диска.
2. Если файловая система поддерживается — перейти в диалоговый режим с ожиданием команд от
   пользователя.

## Поддерживаемые команды

| Команда | Описание |
|---------|----------|
| `LIST` | Вывод списка имён и атрибутов элементов текущей директории |
| `PWD` | Отображение текущей директории |
| `CWD <dir>` | Переход в другую директорию |
| `RETR <file>` | Копирование файла из образа на локальную ФС |
| `QUIT` | Выход |

## Структура Ext3

### Суперблок (Superblock)
- Расположение: байт 1024 от начала раздела, размер 1024 байта
- Магическое число: `0xEF53` (offset 56, 2 байта)
- `s_log_block_size` (offset 24, 4 байта): размер блока = 1024 << s_log_block_size
- `s_inodes_per_group` (offset 40, 4 байта)
- `s_inode_size` (offset 88, 2 байта): размер inode (обычно 128 или 256)

### Block Group Descriptor Table (BGDT)
- Расположение: первый блок после суперблока
- Каждая запись: 32 байта, содержит:
  - `bg_block_bitmap` (offset 0, 4 байта) — блок битовой карты блоков
  - `bg_inode_bitmap` (offset 4, 4 байта) — блок битовой карты inode
  - `bg_inode_table` (offset 8, 4 байта) — начало таблицы inode

### Inode
- Расположение: `bg_inode_table + (inum - 1) / inodes_per_block` блок
- Смещение внутри блока: `((inum - 1) % inodes_per_block) * inode_size`
- `i_mode` (offset 0, 2 байта) — тип и права
- `i_size` (offset 4, 4 байта) — размер файла (младшие 32 бита)
- `i_block[15]` (offset 40, 60 байт) — массив блоков данных

### Directory Entry
- `inode` (offset 0, 4 байта) — номер inode
- `rec_len` (offset 4, 2 байта) — длина записи
- `name_len` (offset 6, 1 байт) — длина имени
- `file_type` (offset 7, 1 байт) — тип: 1 = файл, 2 = директория
- `name` (offset 8, `name_len` байт) — имя

## Как создать Ext3 образ для тестирования

Через WSL (требуется Ubuntu или другой дистрибутив Linux):

```bash
# 1. Создать пустой файл образа
dd if=/dev/zero of=ext3.img bs=1M count=10

# 2. Отформатировать как ext3
mkfs.ext3 -q ext3.img

# 3. Наполнить файлами (через debugfs)
debugfs -w ext3.img -R "mkdir /subdir"
debugfs -w ext3.img -R "write /dev/stdin /hello.txt" <<< "Hello from Ext3!"
debugfs -w ext3.img -R "write /dev/stdin /nested.txt" <<< "Nested file content"
debugfs -w ext3.img -R "write /dev/stdin /subdir/data.txt" <<< "Inside subdir"
```

Образ создаётся в текущей директории, откуда запускается FTP-клиент.

## Сборка и запуск

```powershell
# Компиляция
cargo run -- labs-examples/system-programms/lab-3/input.mylang -o output -t nasm

# Запуск
.\output\program.exe
# Ввести путь к образу: ext3.img
```

## Пример сессии

```
=== Ext3 FTP-like Client ===
Enter Ext3 image path: ext3.img
Ext3 filesystem detected.

ftp> PWD
257 "/"

ftp> LIST
  [DIR]  .
  [DIR]  ..
  [DIR]  lost+found
  [DIR]  subdir
  [FILE] hello.txt
  [FILE] nested.txt

ftp> RETR hello.txt
226 Transfer complete.

ftp> CWD subdir
250 Directory successfully changed.

ftp> PWD
257 "/subdir"

ftp> LIST
  [DIR]  .
  [DIR]  ..
  [FILE] data.txt

ftp> RETR data.txt
226 Transfer complete.

ftp> QUIT
221 Goodbye.
```

Файлы `hello.txt` и `subdir/data.txt` извлечены в текущую директорию.

```
C:\> type hello.txt
Hello from Ext3!

C:\> type data.txt
Inside subdir
```