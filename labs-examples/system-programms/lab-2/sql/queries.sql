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
SELECT tv.name, v.date
FROM type_vedomostei tv
INNER JOIN vedomosti v ON tv.id = v.type_id
WHERE tv.id = 3 AND v.person_id > 153285
ORDER BY v.date;

-- -------------------------------------------------------
-- Query 2: LEFT JOIN with filters
-- Tables: people, studies, students
-- Output: person id, nzk, student id
-- Filter: studies.person_id=163276 AND students.start_date='2008-09-01'
-- -------------------------------------------------------
SELECT '=== Query 2: LEFT JOIN ===';
SELECT p.id, o.nzk, u.person_id
FROM people p
LEFT JOIN studies o ON p.id = o.person_id
LEFT JOIN students u ON p.id = u.person_id
WHERE o.person_id = 163276
  AND u.start_date = '2008-09-01';

-- -------------------------------------------------------
-- Query 3: Count FCE students without patronymic
-- -------------------------------------------------------
SELECT '=== Query 3: Count FCE students without patronymic ===';
SELECT COUNT(*) AS count
FROM people p
INNER JOIN studies o ON p.id = o.person_id
INNER JOIN students u ON p.id = u.person_id
WHERE o.faculty = 'FCE'
  AND (p.patronymic IS NULL OR p.patronymic = '');

-- -------------------------------------------------------
-- Query 4: Plans with >2 groups on CE department
-- Table: group_plans
-- -------------------------------------------------------
SELECT '=== Query 4: Plans >2 groups on CE ===';
SELECT plan_id, COUNT(group_name) AS group_count
FROM group_plans
WHERE department = 'Department of Computer Engineering'
GROUP BY plan_id
HAVING COUNT(group_name) > 2;

-- -------------------------------------------------------
-- Query 5: Avg grades for group 4100 >= avg of group 1100
-- Output: id, full name, avg grade
-- -------------------------------------------------------
SELECT '=== Query 5: Avg grades group 4100 ===';
SELECT
    u.person_id AS id,
    p.surname || ' ' || p.name || COALESCE(' ' || p.patronymic, '') AS full_name,
    ROUND(AVG(CAST(v.grade AS REAL)), 2) AS avg_grade
FROM students u
INNER JOIN people p ON u.person_id = p.id
INNER JOIN vedomosti v ON u.person_id = v.person_id
WHERE u.group_name = '4100'
GROUP BY u.person_id
HAVING AVG(CAST(v.grade AS REAL)) >= (
    SELECT AVG(CAST(v2.grade AS REAL))
    FROM students u2
    INNER JOIN vedomosti v2 ON u2.person_id = v2.person_id
    WHERE u2.group_name = '1100'
)
ORDER BY avg_grade DESC;

-- -------------------------------------------------------
-- Query 6: Enrolled after 2012-09-01, course 1, part-time
-- Subquery with IN
-- -------------------------------------------------------
SELECT '=== Query 6: Enrolled after 2012-09-01, course 1, part-time ===';
SELECT
    u.group_name,
    p.id,
    p.surname || ' ' || p.name || COALESCE(' ' || p.patronymic, '') AS full_name,
    u.order_number,
    u.state
FROM people p
INNER JOIN students u ON p.id = u.person_id
WHERE u.person_id IN (
    SELECT o.person_id
    FROM studies o
    WHERE o.form = 'part-time' AND o.course = 1
)
AND u.start_date > '2012-09-01'
ORDER BY u.group_name, p.surname;

-- -------------------------------------------------------
-- Query 7: Same surname, different birthdays
-- -------------------------------------------------------
SELECT '=== Query 7: Same surname, different birthdays ===';
SELECT p.surname, p.name, COALESCE(p.patronymic, '') AS patronymic, p.birthday
FROM people p
WHERE p.surname IN (
    SELECT surname
    FROM people
    GROUP BY surname
    HAVING COUNT(*) > 1 AND COUNT(DISTINCT birthday) > 1
)
ORDER BY p.surname, p.birthday;
