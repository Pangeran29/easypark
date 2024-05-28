-- Add up migration script here
ALTER TABLE "parking_history" ADD COLUMN     "check_in_date" TIMESTAMP(3),
ADD COLUMN     "check_out_date" TIMESTAMP(3);
