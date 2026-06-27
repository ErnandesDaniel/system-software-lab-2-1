# lab-1: Корутины + Планировщик задач (RR + SRT)

**`input.mylang`** — демо корутин. Две бесконечные корутины печатают `1` и `2` поочерёдно через планировщик.

**`metrics-rr.mylang`** / **`metrics-srt.mylang`** — демо preemptive корутин с RR и SRT шедулерами на mylang.

---

## Сборка и запуск под Linux (WSL)

### Установка Ubuntu

```powershell
wsl --install -d Ubuntu
```

### Установка инструментов

Внутри Ubuntu:

```bash
sudo apt update
sudo apt install -y build-essential nasm curl default-jdk
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### Сборка компилятора

Сначала зайти в WSL (из PowerShell):
```powershell
wsl -d Ubuntu
```

Внутри Ubuntu:
```bash
cd /mnt/c/Users/Ernan/RustroverProjects/system-software-lab-2-1
cargo build --release
```

### Компиляция и запуск под Linux

```bash
cd /mnt/c/Users/Ernan/RustroverProjects/system-software-lab-2-1

# Демо корутин (RR)
cargo run --release -- labs-examples/system-programms/lab-1/input.mylang -o output -t nasm --os linux
./output/program

# Демо RR vs SRT с корутинами
cargo run --release -- labs-examples/system-programms/lab-1/metrics-rr.mylang -o output -t nasm --os linux
./output/program
cargo run --release -- labs-examples/system-programms/lab-1/metrics-srt.mylang -o output -t nasm --os linux
./output/program
```