# lab-1: Компиляция с target под JVM

Базовое демо консольного ввода/вывода и работы со строками, скомпилированное в JVM bytecode.

## Компиляция

### JVM target

```powershell
cargo run -- labs-examples/vitrual-machines/lab-1/input.mylang -o output -t jvm
java -cp output Main
```

### NASM target

```powershell
cargo run -- labs-examples/vitrual-machines/lab-1/input.mylang -o output -t nasm
.\output\program.exe
```
