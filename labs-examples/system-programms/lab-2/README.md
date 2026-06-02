# lab-2: Map-Reduce конвейер (СПО)

Реализация 7 SQL-запросов (вариант 59). Данные читаются из CSV-файлов
в рантайме через `fopen`/`fgetc`. Q1 использует корутины и pipe'ы
(кооперативная многозадачность с passive waiting), Q2-Q7 — прямые циклы.

Все 7 запросов в одном файле `input.mylang`.

## Компиляция и запуск

Запускать из корня проекта (рабочая директория нужна для путей к CSV-файлам).

```powershell
cargo run -- labs-examples/system-programms/lab-2/input.mylang -o output
.\output\program.exe
```

## Файлы

```
labs-examples/system-programms/lab-2/
├── csv-data/                    # CSV-файлы (читаются в рантайме)
│   ├── people.csv               # Н_ЛЮДИ
│   ├── studies.csv              # Н_ОБУЧЕНИЯ
│   ├── students.csv             # Н_УЧЕНИКИ
│   ├── vedomosti.csv            # Н_ВЕДОМОСТИ
│   ├── types_vedomostei.csv     # Н_ТИПЫ_ВЕДОМОСТЕЙ
│   └── group_plans.csv          # Н_ГРУППЫ_ПЛАНОВ
├── sql/                         # Верификация через SQLite
│   ├── queries.sql              # 7 запросов
│   ├── run_verification.cmd     # Батник: создать БД и выполнить запросы
│   └── ucheb_test.db            # Готовая БД
└── input.mylang                 # Все 7 запросов
```

## Ожидаемый вывод

```
=== System Software Lab 2: Map-Reduce Pipeline ===

=== Query 1: INNER JOIN ===
Дифзачет, 2013-06-01
Дифзачет, 2013-06-02
Дифзачет, 2013-06-07
Дифзачет, 2014-01-25

=== Query 2: LEFT JOIN ===
163276, OK500, 163276

=== Query 3: Count FKTИU without patronymic ===
6

=== Query 4: Plans >2 groups on VT ===
101: 3 groups
104: 3 groups

=== Query 5: Avg grades group 4100 >= group 1100 ===
Avg 1100 = 483
Students found: 2

=== Query 6: Enrolled after 2012-09-01, course 1, zaoch ===
Count: 4

=== Query 7: Same surname, different birthdays ===
Surname groups: 12

=== All queries done ===
```

## Верификация через SQLite

```powershell
labs-examples\system-programms\lab-2\sql\run_verification.cmd
```

Результаты запросов:

| # | Результат |
|---|-----------|
| 1 | 4 строки (Дифзачет с датами) |
| 2 | 1 строка (Крылов Кирилл, НЗК=OK500) |
| 3 | 6 (студентов ФКТИУ без отчества) |
| 4 | 2 плана (101: 3 группы, 104: 3 группы) |
| 5 | 2 студента (Григорьев 5.0, Зайцев 5.0) |
| 6 | 4 студента (заочные 1 курс после 2012) |
| 7 | 12 строк (6 пар с одинаковыми фамилиями) |
