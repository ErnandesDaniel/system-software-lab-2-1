# Lab 2: Map-Reduce Pipeline (SPO)

Implementation of 7 SQL queries (variant 59). Data is read from CSV files
at runtime via `fopen`/`fgetc`. All 7 queries use direct loops.

All 7 queries in one file `input.mylang`.

## Compile & Run

Run from the project root (working directory matters for CSV paths).

```powershell
cargo run -- labs-examples/system-programms/lab-2/input.mylang -o output
.\output\program.exe
```

## Files

```
labs-examples/system-programms/lab-2/
├── csv-data/                    # CSV files (read at runtime)
│   ├── people.csv
│   ├── studies.csv
│   ├── students.csv
│   ├── vedomosti.csv
│   ├── types_vedomostei.csv
│   └── group_plans.csv
├── sql/                         # SQLite verification
│   ├── queries.sql              # 7 queries
│   ├── run_verification.cmd     # Batch: create DB + run queries
│   └── ucheb_test.db
└── input.mylang                 # All 7 queries
```

## Expected output

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

## SQLite verification

```powershell
labs-examples\system-programms\lab-2\sql\run_verification.cmd
```

Results:

| # | Result |
|---|--------|
| 1 | 4 rows (DiffPass with dates) |
| 2 | 1 row (163276, OK500, 163276) |
| 3 | 6 (FCE students without patronymic) |
| 4 | 2 plans (101: 3 groups, 104: 3 groups) |
| 5 | 2 students (Grigoriev 5.0, Zaitsev 5.0) |
| 6 | 4 students (part-time, course 1, after 2012) |
| 7 | 12 rows (6 pairs with same surname) |
