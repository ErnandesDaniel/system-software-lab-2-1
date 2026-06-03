-- =====================================================
-- Lab 2 — SQL verification queries
-- Variant 59
-- =====================================================

-- -------------------------------------------------------
-- Query 1: INNER JOIN with filters
-- Tables: type_vedomostei, vedomosti
-- Output: type name, date
-- Filter: tv.id=3 (DiffPass) AND v.person_id > 153285
-- -------------------------------------------------------
SELECT '=== Query 1: INNER JOIN ===';
SELECT tv.НАИМЕНОВАНИЕ, v.ДАТА
FROM Н_ТИПЫ_ВЕДОМОСТЕЙ tv
INNER JOIN Н_ВЕДОМОСТИ v ON tv.ИД = v.ТИП_ИД
WHERE tv.ИД = 3 AND v.ЧЛВК_ИД > 153285
ORDER BY v.ДАТА;

-- -------------------------------------------------------
-- Query 2: LEFT JOIN with filters
-- Tables: people, studies, students
-- Output: person id, nzk, student id
-- Filter: studies.person_id=163276 AND students.start_date='2008-09-01'
-- -------------------------------------------------------
SELECT '=== Query 2: LEFT JOIN ===';
SELECT p.ИД, o.НЗК, u.ИД
FROM Н_ЛЮДИ p
LEFT JOIN Н_ОБУЧЕНИЯ o ON p.ИД = o.ЧЛВК_ИД
LEFT JOIN Н_УЧЕНИКИ u ON p.ИД = u.ИД
WHERE o.ЧЛВК_ИД = 163276
  AND u.НАЧАЛО = '2008-09-01';

-- -------------------------------------------------------
-- Query 3: Count FCE students without patronymic
-- -------------------------------------------------------
SELECT '=== Query 3: Count FCE students without patronymic ===';
SELECT COUNT(*) AS count
FROM Н_ЛЮДИ p
INNER JOIN Н_ОБУЧЕНИЯ o ON p.ИД = o.ЧЛВК_ИД
INNER JOIN Н_УЧЕНИКИ u ON p.ИД = u.ИД
WHERE o.ФАКУЛЬТЕТ = 'FCE'
  AND (p.ОТЧЕСТВО IS NULL OR p.ОТЧЕСТВО = '');

-- -------------------------------------------------------
-- Query 4: Plans with >2 groups on CE department
-- Table: group_plans
-- -------------------------------------------------------
SELECT '=== Query 4: Plans >2 groups on CE ===';
SELECT ПЛАН_ИД AS plan_id, COUNT(ГРУППА) AS group_count
FROM Н_ГРУППЫ_ПЛАНОВ
WHERE КАФЕДРА = 'Department of Computer Engineering'
GROUP BY ПЛАН_ИД
HAVING COUNT(ГРУППА) > 2;

-- -------------------------------------------------------
-- Query 5: Avg grades for group 4100 >= avg of group 1100
-- Output: id, full name, avg grade
-- -------------------------------------------------------
SELECT '=== Query 5: Avg grades group 4100 ===';
SELECT
    u.ИД AS id,
    p.ФАМИЛИЯ || ' ' || p.ИМЯ || COALESCE(' ' || p.ОТЧЕСТВО, '') AS full_name,
    ROUND(AVG(CAST(v.ОЦЕНКА AS REAL)), 2) AS avg_grade
FROM Н_УЧЕНИКИ u
INNER JOIN Н_ЛЮДИ p ON u.ИД = p.ИД
INNER JOIN Н_ВЕДОМОСТИ v ON u.ИД = v.ЧЛВК_ИД
WHERE u.ГРУППА = '4100'
GROUP BY u.ИД
HAVING AVG(CAST(v.ОЦЕНКА AS REAL)) >= (
    SELECT AVG(CAST(v2.ОЦЕНКА AS REAL))
    FROM Н_УЧЕНИКИ u2
    INNER JOIN Н_ВЕДОМОСТИ v2 ON u2.ИД = v2.ЧЛВК_ИД
    WHERE u2.ГРУППА = '1100'
)
ORDER BY avg_grade DESC;

-- -------------------------------------------------------
-- Query 6: Enrolled after 2012-09-01, course 1, part-time
-- Subquery with IN
-- -------------------------------------------------------
SELECT '=== Query 6: Enrolled after 2012-09-01, course 1, part-time ===';
SELECT
    u.ГРУППА,
    p.ИД,
    p.ФАМИЛИЯ || ' ' || p.ИМЯ || COALESCE(' ' || p.ОТЧЕСТВО, '') AS full_name,
    u.ПРИКАЗ,
    u.СОСТОЯНИЕ
FROM Н_ЛЮДИ p
INNER JOIN Н_УЧЕНИКИ u ON p.ИД = u.ИД
WHERE u.ИД IN (
    SELECT o.ЧЛВК_ИД
    FROM Н_ОБУЧЕНИЯ o
    WHERE o.ФОРМА = 'part-time' AND o.КУРС = 1
)
AND u.НАЧАЛО > '2012-09-01'
ORDER BY u.ГРУППА, p.ФАМИЛИЯ;

-- -------------------------------------------------------
-- Query 7: Same surname, different birthdays
-- -------------------------------------------------------
SELECT '=== Query 7: Same surname, different birthdays ===';
SELECT p.ФАМИЛИЯ, p.ИМЯ, COALESCE(p.ОТЧЕСТВО, '') AS patronymic, p.ДАТА_РОЖДЕНИЯ
FROM Н_ЛЮДИ p
WHERE p.ФАМИЛИЯ IN (
    SELECT ФАМИЛИЯ
    FROM Н_ЛЮДИ
    GROUP BY ФАМИЛИЯ
    HAVING COUNT(*) > 1 AND COUNT(DISTINCT ДАТА_РОЖДЕНИЯ) > 1
)
ORDER BY p.ФАМИЛИЯ, p.ДАТА_РОЖДЕНИЯ;
