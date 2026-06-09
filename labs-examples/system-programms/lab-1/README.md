
Использование прерывания по таймеру
signal api

sigaction библиотека
signal.h

# lab-1: Корутины + Планировщик задач (RR + SRT)

**`input.mylang`** — демо корутин. Две бесконечные корутины печатают `1` и `2` поочерёдно через планировщик.

**`metrics.mylang`** — симуляция алгоритмов планирования Round Robin (квант 2) и Shortest Remaining Time.

---

## Сборка и запуск под Linux (WSL)

### 1. Установка Ubuntu в WSL

В **PowerShell**:

```powershell
wsl --install -d Ubuntu
```

Запустите `wsl -d Ubuntu`, создайте пользователя и пароль.

### 2. Установка инструментов внутри Ubuntu

Все следующие команды выполняются **в терминале Ubuntu** (не PowerShell):

```bash
# Обновить пакеты
sudo apt update

# Установить компилятор C, NASM, curl
sudo apt install -y build-essential nasm curl default-jdk

# Установить Rust (без интерактива)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Добавить cargo в PATH для текущей сессии
source "$HOME/.cargo/env"
```

### 3. Сборка компилятора

```bash
# Перейти в директорию проекта (Windows-диск монтируется в /mnt/c/)
cd /mnt/c/Users/Ernan/RustroverProjects/system-software-lab-2-1

# Собрать компилятор
cargo build --release
```

### 4. Компиляция и запуск .mylang

```bash
# input.mylang — демо корутин
cargo run --release -- labs-examples/system-programms/lab-1/input.mylang -o output -t nasm --os linux
./output/program

# metrics.mylang — симуляция планировщика
cargo run --release -- labs-examples/system-programms/lab-1/metrics.mylang -o output -t nasm --os linux
./output/program
```

**Ключ `--os linux`** переключает кодогенерацию:
- Формат: `elf64` (вместо `win64`)
- Calling convention: System V AMD64 (rdi, rsi, rdx, rcx, r8, r9)
- Линковка: `gcc -no-pie`
- Preemptive корутины через `setitimer` + `sigaction(SIGVTALRM)` + `ucontext_t`

---

## Сборка и запуск под Windows

В **PowerShell** (из корня проекта):

```powershell
cargo run -- labs-examples/system-programms/lab-1/input.mylang -o output -t nasm
.\output\program.exe
```
```powershell
cargo run -- labs-examples/system-programms/lab-1/input.mylang -o output -t jvm
java -cp output Main
```
---

```powershell
cargo run -- labs-examples/system-programms/lab-1/metrics.mylang -o output -t nasm
.\output\program.exe
```
```powershell
cargo run -- labs-examples/system-programms/lab-1/metrics.mylang -o output -t jvm
java -cp output Main
```

## Что выводит `metrics.mylang`

Программа делает два теста — с разной средней частотой поступления процессов:

### 1. Таблица процессов

Для каждого процесса (P00–P19) печатается:
```
  # | arrive | burst
P00 @    0 b   5
P01 @   11 b   6
P02 @   13 b   4
...
```
- **`arrive`** — время появления процесса (в условных единицах времени)
- **`burst`** — длина CPU burst (сколько времени процессу нужно проработать на процессоре)

Во втором тесте процессы приходят чаще (средний интервал 3 вместо 6), поэтому они сильнее пересекаются и создают бóльшую конкуренцию за CPU.

### 2. Метрики: средние turnaround и wait

Для каждого алгоритма (RR(2) и SRT) выводятся две усреднённые метрики:

```
RR(2):
  Avg turn: 71
  Avg wait: 65

SRT:
  Avg turn: 63
  Avg wait: 57
```

---

## Что означают метрики

- **Turnaround time** (время оборота) — сколько прошло от момента появления процесса (`arrival`) до его завершения (`finish_time`). Это полное время жизни процесса в системе, включая ожидание в очереди и собственно выполнение.  
  `turnaround[i] = finish_time[i] - arrival[i]`

- **Wait time** (время ожидания) — сколько процесс простоял в очереди, не получая CPU. Рассчитывается как разность между turnaround и временем, реально потраченным на вычисления (burst):  
  `wait[i] = turnaround[i] - burst[i]`

Чем меньше эти средние — тем эффективнее алгоритм планирования.

---

## Как симуляция устроена

1. **Генерация процессов** — `gen_procs(n, mean_ia)` создаёт `n` процессов:
   - `arrival[0] = 0`, каждый следующий приходит через случайный интервал со средним `mean_ia` (равномерное распределение от 1 до `2×mean_ia`)
   - `burst` — случайное целое от 4 до 8

2. **RR(2) (Round Robin с квантом 2)** — круговой алгоритм: каждый процесс получает CPU не более чем на 2 единицы времени, после чего уступает следующему из очереди FIFO.

3. **SRT (Shortest Remaining Time)** — вытесняющий алгоритм: в каждый момент времени выполняется процесс с наименьшим оставшимся burst-ом. Если приходит новый процесс с меньшим остатком, текущий вытесняется.

4. **Сброс состояния** — для второго теста сначала генерируется новый набор процессов с `mean_ia = 3`, затем оба алгоритма прогоняются заново (`reset_state`). Сид генератора случайных чисел фиксирован (`srand(42)`), поэтому результаты воспроизводимы.

---

## Ожидаемая картина

- RR(2) даёт бóльший средний turnaround и wait, чем SRT, потому что:
  - Короткие процессы вынуждены ждать кванта времени наравне с длинными
  - Переключения контекста (прерывания по кванту) добавляют задержки
- SRT минимизирует среднее время ожидания, давая коротким процессам выполняться сразу
- Во втором тесте (интервал 3) средние метрики выше, чем в первом, так как процессы накладываются друг на друга сильнее

---

## Технические детали (Linux preemptive корутины)

Preemptive многозадачность реализована через:

- `setitimer(ITIMER_VIRTUAL, ...)` — таймер, считающий CPU time в user-mode
- `sigaction(SIGVTALRM, ...)` с флагом `SA_NODEFER` — обработчик прерывания
- `ucontext_t` — сохранение/восстановление регистров (RIP, RSP, RBX, RBP, R12-R15) прямо в обработчике сигнала
- После возврата из обработчика ядро продолжает выполнение с новым контекстом
