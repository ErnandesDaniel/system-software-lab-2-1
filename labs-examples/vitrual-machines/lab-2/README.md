# lab-2: Функции первого класса + замыкания

Демонстрация локальных функций, функциональных литералов, композиции, замыканий (read-only и mutate) и счётчика.

## Компиляция

### JVM target

```powershell
cargo run -- labs-examples/vitrual-machines/lab-2/input.mylang -o output -t jvm
java -cp output Main
```

### NASM target

```powershell
cargo run -- labs-examples/vitrual-machines/lab-2/input.mylang -o output -t nasm
.\output\program.exe
```

В одном файле 7 сценариев:

| # | Сценарий | Результат |
|---|----------|-----------|
| 1 | Локальная `double(21)` | `42` |
| 2 | Функциональный литерал `square(5)` | `25` |
| 3 | Композиция `apply_twice(double, 3)` | `12` |
| 4 | Замыкание (read-only) `read_x()` | `10` |
| 5 | Замыкание (мутация) `inc_y()` ×3 | `y = 3` |
| 6 | Счётчик через замыкание `inc_count()` ×3 | `123` |
| 7 | Комбинация всего | `2730` |

Надо исправить так, чтобы состояние вне двух функций изменялось после использования этих функций типа

написать нормальный тип тут и проверить как тут, так и на PHP
def f() of def(int) of int array[2] {
y = 0;

    return [
        def add2(x of int) of int {
            y = x + y; 
            return y;
        },
        def mul2(x of int) of int {
            y = x * y;
            return y;
        }
    ];
}

вот тут переменная `y` будет меняться типа


переписать на PHP для 4 лабы и проверить что оно будет работать
def test() of int {
c1 = f();
c2 = f();

    x1 = c1[0](2);
    printf("c1[0](2) = %d\n", x1);

    y1 = c2[0](2);
    printf("c2[0](2) = %d\n", y1);

    x2 = c1[0](2);
    printf("c1[0](2) = %d\n", x2);

    y2 = c2[1](7);
    printf("c2[1](7) = %d\n", y2);

    x3 = c1[1](7);
    printf("c1[1](7) = %d\n", x3);

    y3 = c2[0](3);
    printf("c2[0](3) = %d\n", y3);

    return (x1 + x2 + x3) * 100 + (y1 + y2 + y3);
}

