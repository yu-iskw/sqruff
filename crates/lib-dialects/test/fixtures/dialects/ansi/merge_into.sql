-- Merge using Table
MERGE INTO t USING u ON (a = b)
WHEN MATCHED THEN
UPDATE SET a = b
WHEN NOT MATCHED THEN
INSERT (b) VALUES (c);

-- Merge using Select
MERGE INTO t USING (SELECT * FROM u) AS u ON (a = b)
WHEN MATCHED THEN
UPDATE SET a = b
WHEN NOT MATCHED THEN
INSERT (b) VALUES (c);

-- Merge using Delete
MERGE INTO t USING u ON (a = b)
WHEN MATCHED THEN
UPDATE SET a = b
WHEN MATCHED THEN DELETE;

-- Merge using multiple operations
MERGE INTO t USING u ON (a = b)
WHEN MATCHED AND a > b THEN
UPDATE SET a = b
WHEN MATCHED AND ( a < b AND c < d ) THEN DELETE
WHEN NOT MATCHED THEN INSERT (a, c) VALUES (b, d);

-- Merge using CTE
WITH source AS (
    SELECT
        *
    FROM u
)
MERGE INTO t USING source AS u ON (a = b)
WHEN MATCHED THEN
UPDATE SET a = b
WHEN NOT MATCHED THEN
INSERT (b) VALUES (c);