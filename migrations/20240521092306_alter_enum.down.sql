-- Step 1: Create temporary enums without 'default' value

CREATE TYPE role_temp AS ENUM ('easypark', 'park_keeper', 'park_owner');
CREATE TYPE user_status_temp AS ENUM ('active', 'not_active');
CREATE TYPE ticket_status_temp AS ENUM ('active', 'not_active');
CREATE TYPE vehicle_type_temp AS ENUM ('car', 'motor');
CREATE TYPE payment_type_temp AS ENUM ('cash', 'qr');

-- Step 2: Alter tables to use temporary enums
-- Replace table_name and column_name with actual table and column names.

ALTER TABLE table_name ALTER COLUMN role TYPE role_temp USING role::text::role_temp;
ALTER TABLE table_name ALTER COLUMN user_status TYPE user_status_temp USING user_status::text::user_status_temp;
ALTER TABLE table_name ALTER COLUMN ticket_status TYPE ticket_status_temp USING ticket_status::text::ticket_status_temp;
ALTER TABLE table_name ALTER COLUMN vehicle_type TYPE vehicle_type_temp USING vehicle_type::text::vehicle_type_temp;
ALTER TABLE table_name ALTER COLUMN payment_type TYPE payment_type_temp USING payment_type::text::payment_type_temp;

-- Step 3: Drop original enums

DROP TYPE role;
DROP TYPE user_status;
DROP TYPE ticket_status;
DROP TYPE vehicle_type;
DROP TYPE payment_type;

-- Step 4: Recreate original enums without 'default' value

CREATE TYPE role AS ENUM ('easypark', 'park_keeper', 'park_owner');
CREATE TYPE user_status AS ENUM ('active', 'not_active');
CREATE TYPE ticket_status AS ENUM ('active', 'not_active');
CREATE TYPE vehicle_type AS ENUM ('car', 'motor');
CREATE TYPE payment_type AS ENUM ('cash', 'qr');

-- Step 5: Alter tables to use recreated original enums

ALTER TABLE table_name ALTER COLUMN role TYPE role USING role::text::role;
ALTER TABLE table_name ALTER COLUMN user_status TYPE user_status USING user_status::text::user_status;
ALTER TABLE table_name ALTER COLUMN ticket_status TYPE ticket_status USING ticket_status::text::ticket_status;
ALTER TABLE table_name ALTER COLUMN vehicle_type TYPE vehicle_type USING vehicle_type::text::vehicle_type;
ALTER TABLE table_name ALTER COLUMN payment_type TYPE payment_type USING payment_type::text::payment_type;

-- Step 6: Drop temporary enums

DROP TYPE role_temp;
DROP TYPE user_status_temp;
DROP TYPE ticket_status_temp;
DROP TYPE vehicle_type_temp;
DROP TYPE payment_type_temp;
