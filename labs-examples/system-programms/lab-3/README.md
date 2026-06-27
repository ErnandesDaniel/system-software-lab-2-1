# Lab 3 — FTP-like CLI для Ext3

## Как создать Ext3 образ для тестирования

Через WSL:

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

## Сборка и запуск под Linux (WSL)

Запускать из корня проекта.

Сначала зайти в WSL (из PowerShell):
```powershell
wsl -d Ubuntu
```

```bash
cd /mnt/c/Users/Ernan/RustroverProjects/system-software-lab-2-1

# Компиляция
cargo run --release -- labs-examples/system-programms/lab-3/input.mylang -o output -t nasm --os linux

# Запуск
./output/program
# Ввести путь к образу: ext3.img
# Ввести путь к образу: 
ext3.bin
```

Файлы `hello.txt` и `subdir/data.txt` извлечены в текущую директорию.

```
C:\> type hello.txt
Hello from Ext3!

C:\> type data.txt
Inside subdir
```

ls -la ext3_bin

## Работа с ext3.bin через WSL

Файл `ext3.bin` в корне проекта — это образ ext3. Для просмотра содержимого смонтируйте его в папку `ext3_bin/` (уже существует в проекте):

```bash
# Из корня проекта:
cd /mnt/c/Users/Ernan/RustroverProjects/system-software-lab-2-1

# 1. Создать точку монтирования
sudo mkdir -p ext3_mnt

# 2. Смонтировать образ
sudo mount -o loop ext3.bin ext3_mnt

# 3. Просмотр содержимого
ls -la ext3_mnt

# 4. После работы — отмонтировать
sudo umount ext3_mnt

# 5. Опционально — удалить точку монтирования
sudo rmdir ext3_mnt
```