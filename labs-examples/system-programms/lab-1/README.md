# lab-1: Корутины + Планировщик задач (RR + SRT)

Два файла:

**`input.mylang`** — демо корутин. Две бесконечные корутины печатают `1` и `2` поочерёдно через планировщик.

```powershell
cargo run -- labs-examples/system-programms/lab-1/input.mylang -o output -t nasm
.\output\program.exe
```

**`metrics.mylang`** — симулятор планировщика (вариант 19: RR(2) + SRT). Диапазон burst 4–8, средние интервалы 6 и 3. Выводит таблицу процессов и средние turnaround/wait для каждого алгоритма.

```powershell
cargo run -- labs-examples/system-programms/lab-1/metrics.mylang -o output -t nasm
.\output\program.exe
```
