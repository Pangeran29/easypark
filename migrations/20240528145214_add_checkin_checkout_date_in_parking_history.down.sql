-- Remove the columns added in the up migration
ALTER TABLE "parking_history" DROP COLUMN "check_in_date",
DROP COLUMN "check_out_date";