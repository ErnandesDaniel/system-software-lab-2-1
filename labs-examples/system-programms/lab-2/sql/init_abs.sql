-- =====================================================
-- Создание таблиц с русскими названиями (как в задании)
-- =====================================================

CREATE TABLE IF NOT EXISTS Н_ТИПЫ_ВЕДОМОСТЕЙ (
    ИД INTEGER PRIMARY KEY,
    НАИМЕНОВАНИЕ TEXT
);

CREATE TABLE IF NOT EXISTS Н_ВЕДОМОСТИ (
    ИД INTEGER PRIMARY KEY,
    ТИП_ИД INTEGER,
    ЧЛВК_ИД INTEGER,
    ДАТА TEXT,
    ОЦЕНКА INTEGER
);

CREATE TABLE IF NOT EXISTS Н_ЛЮДИ (
    ИД INTEGER PRIMARY KEY,
    ФАМИЛИЯ TEXT,
    ИМЯ TEXT,
    ОТЧЕСТВО TEXT,
    ДАТА_РОЖДЕНИЯ TEXT
);

CREATE TABLE IF NOT EXISTS Н_ОБУЧЕНИЯ (
    ЧЛВК_ИД INTEGER,
    НЗК TEXT,
    ФОРМА TEXT,
    НАПР_КОД TEXT,
    КАФЕДРА TEXT,
    ФАКУЛЬТЕТ TEXT,
    КУРС INTEGER
);

CREATE TABLE IF NOT EXISTS Н_УЧЕНИКИ (
    ИД INTEGER,
    ГРУППА TEXT,
    НАЧАЛО TEXT,
    КОНЕЦ TEXT,
    ПРИКАЗ TEXT,
    СОСТОЯНИЕ TEXT,
    СТАТУС TEXT
);

CREATE TABLE IF NOT EXISTS Н_ГРУППЫ_ПЛАНОВ (
    ПЛАН_ИД INTEGER,
    ГРУППА TEXT,
    КАФЕДРА TEXT
);

-- =====================================================
-- Импорт CSV-данных
-- =====================================================

.mode csv

-- Н_ТИПЫ_ВЕДОМОСТЕЙ
CREATE TABLE IF NOT EXISTS _tv (id TEXT, name TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/types_vedomostei.csv _tv
INSERT INTO Н_ТИПЫ_ВЕДОМОСТЕЙ (ИД, НАИМЕНОВАНИЕ)
    SELECT CAST(id AS INTEGER), name FROM _tv WHERE id != 'id';
DROP TABLE _tv;

-- Н_ЛЮДИ
CREATE TABLE IF NOT EXISTS _ppl (id TEXT, surname TEXT, name TEXT, patronymic TEXT, birthday TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/people.csv _ppl
INSERT INTO Н_ЛЮДИ (ИД, ФАМИЛИЯ, ИМЯ, ОТЧЕСТВО, ДАТА_РОЖДЕНИЯ)
    SELECT CAST(id AS INTEGER), surname, name, NULLIF(patronymic, ''), birthday
    FROM _ppl WHERE id != 'id';
DROP TABLE _ppl;

-- Н_ОБУЧЕНИЯ
CREATE TABLE IF NOT EXISTS _std (person_id TEXT, nzk TEXT, form TEXT, direction_code TEXT, department TEXT, faculty TEXT, course TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/studies.csv _std
INSERT INTO Н_ОБУЧЕНИЯ (ЧЛВК_ИД, НЗК, ФОРМА, НАПР_КОД, КАФЕДРА, ФАКУЛЬТЕТ, КУРС)
    SELECT CAST(person_id AS INTEGER), nzk, form, direction_code, department, faculty, CAST(course AS INTEGER)
    FROM _std WHERE person_id != 'person_id';
DROP TABLE _std;

-- Н_ВЕДОМОСТИ
CREATE TABLE IF NOT EXISTS _vd (id TEXT, type_id TEXT, person_id TEXT, date TEXT, mark TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/vedomosti.csv _vd
INSERT INTO Н_ВЕДОМОСТИ (ИД, ТИП_ИД, ЧЛВК_ИД, ДАТА, ОЦЕНКА)
    SELECT CAST(id AS INTEGER), CAST(type_id AS INTEGER), CAST(person_id AS INTEGER), date, CAST(mark AS INTEGER)
    FROM _vd WHERE id != 'id';
DROP TABLE _vd;

-- Н_УЧЕНИКИ
CREATE TABLE IF NOT EXISTS _st (person_id TEXT, group_name TEXT, start_date TEXT, end_date TEXT, order_number TEXT, order_state TEXT, status TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/students.csv _st
INSERT INTO Н_УЧЕНИКИ (ИД, ГРУППА, НАЧАЛО, КОНЕЦ, ПРИКАЗ, СОСТОЯНИЕ, СТАТУС)
    SELECT CAST(person_id AS INTEGER), group_name, start_date, NULLIF(end_date, ''), order_number, order_state, status
    FROM _st WHERE person_id != 'person_id';
DROP TABLE _st;

-- Н_ГРУППЫ_ПЛАНОВ
CREATE TABLE IF NOT EXISTS _gp (plan_id TEXT, group_name TEXT, department TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/group_plans.csv _gp
INSERT INTO Н_ГРУППЫ_ПЛАНОВ (ПЛАН_ИД, ГРУППА, КАФЕДРА)
    SELECT CAST(plan_id AS INTEGER), group_name, department
    FROM _gp WHERE plan_id != 'plan_id';
DROP TABLE _gp;

.headers on
.mode column
SELECT 'Импорт завершён.';
SELECT 'Н_ТИПЫ_ВЕДОМОСТЕЙ:', COUNT(*) FROM Н_ТИПЫ_ВЕДОМОСТЕЙ;
SELECT 'Н_ЛЮДИ:', COUNT(*) FROM Н_ЛЮДИ;
SELECT 'Н_ОБУЧЕНИЯ:', COUNT(*) FROM Н_ОБУЧЕНИЯ;
SELECT 'Н_ВЕДОМОСТИ:', COUNT(*) FROM Н_ВЕДОМОСТИ;
SELECT 'Н_УЧЕНИКИ:', COUNT(*) FROM Н_УЧЕНИКИ;
SELECT 'Н_ГРУППЫ_ПЛАНОВ:', COUNT(*) FROM Н_ГРУППЫ_ПЛАНОВ;

