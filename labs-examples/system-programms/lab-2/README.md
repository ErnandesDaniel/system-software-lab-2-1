# Лаба 2: Map-Reduce конвейер (СПО)

Реализация 7 SQL-запросов (вариант 59). Данные читаются из CSV-файлов
в рантайме через `fopen`/`fgetc`. Все 7 запросов — прямые циклы.

Все запросы в одном файле `input.mylang`.

## Компиляция и запуск под Linux (WSL)

Запускать из корня проекта (рабочая директория нужна для путей к CSV).

```bash
cd /mnt/c/Users/Ernan/RustroverProjects/system-software-lab-2-1

# NASM target
cargo run --release -- labs-examples/system-programms/lab-2/input.mylang -o output -t nasm --os linux
./output/program

# JVM target
cargo run --release -- labs-examples/system-programms/lab-2/input.mylang -o output -t jvm --os linux
java -cp output Main
```

## Файлы

```
labs-examples/system-programms/lab-2/
├── csv-data/                    # CSV-файлы (читаются в рантайме)
│   ├── people.csv
│   ├── studies.csv
│   ├── students.csv
│   ├── vedomosti.csv
│   ├── types_vedomostei.csv
│   └── group_plans.csv
├── sql/                         # Верификация через SQLite
│   ├── queries.sql              # 7 запросов
│   ├── run_verification.cmd     # Батник: создать БД + выполнить запросы
│   └── ucheb_test.db
└── input.mylang                 # Все 7 запросов
```

## Ожидаемый вывод

```
=== Lab 2: Map-Reduce Pipeline ===

=== Q1: INNER JOIN ===
DiffPass, 2013-06-01
DiffPass, 2013-06-07
DiffPass, 2013-06-02
DiffPass, 2014-01-25
Found: 4

=== Q2: LEFT JOIN ===
163276, OK500, 163276
Found: 1

=== Q3: Count FCE without patronymic ===
6

=== Q4: Plans >2 groups on CE ===
101: 3 groups
104: 3 groups

=== Q5: Avg grades 4100 >= 1100 ===
Avg 1100 = 48
100010, Zaitsev Zakhar 5.0
100014, Grigoriev Georgy 5.0
Found: 2

=== Q6: Enrolled after 2012-09-01, 1 course, part-time ===
4100, 100015, Timofeev Timur
2100, 100016, Zhukov Zhan
2100, 200001, Sokolov Maksim
2100, 210001, Belov Ivan
Count: 4

=== Q7: Same surname, diff bday ===
Morozov, Dmitry, 2005-08-08
Morozov, Alexey, 2004-12-01
Novikov, Nikolay, 2005-01-01
Zhukov, Zhan, 2001-12-12
Zhukov, Zhenya, 2003-08-18
Krylov, Kirill, 2003-09-09
Novikov, Stepan, 2004-12-12
Sokolov, Maksim, 2003-03-03
Sokolov, Vitaly, 2005-05-05
Belov, Ivan, 2004-04-04
Belov, Sergey, 2002-02-02
Krylov, Denis, 2001-01-01
Groups: 12

=== Done ===
```

## Верификация через SQLite

```powershell
# Windows
labs-examples\system-programms\lab-2\sql\run_verification.cmd
```

```bash
# Linux (WSL)
sqlite3 :memory: -init labs-examples/system-programms/lab-2/sql/init.sql \
  ".read labs-examples/system-programms/lab-2/sql/queries.sql" 2>&1 | grep -v "^--"
```

Results:

| # | Результат |
|---|--------|
| 1 | 4 rows (DiffPass with dates) |
| 2 | 1 row (163276, OK500, 163276) |
| 3 | 6 (FCE students without patronymic) |
| 4 | 2 plans (101: 3 groups, 104: 3 groups) |
| 5 | 2 students (Grigoriev 5.0, Zaitsev 5.0) |
| 6 | 4 students (part-time, course 1, after 2012) |
| 7 | 12 rows (6 pairs with same surname) |
