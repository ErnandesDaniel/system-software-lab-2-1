-- =====================================================
-- Create tables (English names)
-- =====================================================

CREATE TABLE IF NOT EXISTS type_vedomostei (
    id INTEGER PRIMARY KEY,
    name TEXT
);

CREATE TABLE IF NOT EXISTS vedomosti (
    id INTEGER PRIMARY KEY,
    type_id INTEGER,
    person_id INTEGER,
    date TEXT,
    grade INTEGER
);

CREATE TABLE IF NOT EXISTS people (
    id INTEGER PRIMARY KEY,
    surname TEXT,
    name TEXT,
    patronymic TEXT,
    birthday TEXT
);

CREATE TABLE IF NOT EXISTS studies (
    person_id INTEGER,
    nzk TEXT,
    form TEXT,
    direction_code TEXT,
    department TEXT,
    faculty TEXT,
    course INTEGER
);

CREATE TABLE IF NOT EXISTS students (
    person_id INTEGER,
    group_name TEXT,
    start_date TEXT,
    end_date TEXT,
    order_number TEXT,
    state TEXT,
    status TEXT
);

CREATE TABLE IF NOT EXISTS group_plans (
    plan_id INTEGER,
    group_name TEXT,
    department TEXT
);

-- =====================================================
-- Import CSV data
-- =====================================================

.mode csv

-- types_vedomostei
CREATE TABLE IF NOT EXISTS _tv (id TEXT, name TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/types_vedomostei.csv _tv
INSERT INTO type_vedomostei (id, name)
    SELECT CAST(id AS INTEGER), name FROM _tv WHERE id != 'id';
DROP TABLE _tv;

-- people
CREATE TABLE IF NOT EXISTS _ppl (id TEXT, surname TEXT, name TEXT, patronymic TEXT, birthday TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/people.csv _ppl
INSERT INTO people (id, surname, name, patronymic, birthday)
    SELECT CAST(id AS INTEGER), surname, name, NULLIF(patronymic, ''), birthday
    FROM _ppl WHERE id != 'id';
DROP TABLE _ppl;

-- studies
CREATE TABLE IF NOT EXISTS _std (person_id TEXT, nzk TEXT, form TEXT, direction_code TEXT, department TEXT, faculty TEXT, course TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/studies.csv _std
INSERT INTO studies (person_id, nzk, form, direction_code, department, faculty, course)
    SELECT CAST(person_id AS INTEGER), nzk, form, direction_code, department, faculty, CAST(course AS INTEGER)
    FROM _std WHERE person_id != 'person_id';
DROP TABLE _std;

-- vedomosti
CREATE TABLE IF NOT EXISTS _vd (id TEXT, type_id TEXT, person_id TEXT, date TEXT, grade TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/vedomosti.csv _vd
INSERT INTO vedomosti (id, type_id, person_id, date, grade)
    SELECT CAST(id AS INTEGER), CAST(type_id AS INTEGER), CAST(person_id AS INTEGER), date, CAST(grade AS INTEGER)
    FROM _vd WHERE id != 'id';
DROP TABLE _vd;

-- students
CREATE TABLE IF NOT EXISTS _st (person_id TEXT, group_name TEXT, start_date TEXT, end_date TEXT, order_number TEXT, state TEXT, status TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/students.csv _st
INSERT INTO students (person_id, group_name, start_date, end_date, order_number, state, status)
    SELECT CAST(person_id AS INTEGER), group_name, start_date, NULLIF(end_date, ''), order_number, state, status
    FROM _st WHERE person_id != 'person_id';
DROP TABLE _st;

-- group_plans
CREATE TABLE IF NOT EXISTS _gp (plan_id TEXT, group_name TEXT, department TEXT);
.import C:/Users/Ernan/RustroverProjects/system-software-lab-2-1/labs-examples/system-programms/lab-2/csv-data/group_plans.csv _gp
INSERT INTO group_plans (plan_id, group_name, department)
    SELECT CAST(plan_id AS INTEGER), group_name, department
    FROM _gp WHERE plan_id != 'plan_id';
DROP TABLE _gp;

.headers on
.mode column
SELECT 'Import complete.';
SELECT 'type_vedomostei:', COUNT(*) FROM type_vedomostei;
SELECT 'people:', COUNT(*) FROM people;
SELECT 'studies:', COUNT(*) FROM studies;
SELECT 'vedomosti:', COUNT(*) FROM vedomosti;
SELECT 'students:', COUNT(*) FROM students;
SELECT 'group_plans:', COUNT(*) FROM group_plans;
