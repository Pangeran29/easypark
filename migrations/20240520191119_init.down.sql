-- DropForeignKey
ALTER TABLE "parking_history" DROP CONSTRAINT "parking_history_transaction_id_fkey";
ALTER TABLE "parking_history" DROP CONSTRAINT "parking_history_owner_id_fkey";
ALTER TABLE "parking_history" DROP CONSTRAINT "parking_history_keeper_id_fkey";
ALTER TABLE "parking_history" DROP CONSTRAINT "parking_history_easypark_id_fkey";
ALTER TABLE "parking_history" DROP CONSTRAINT "parking_history_parking_lot_id_fkey";
ALTER TABLE "parking_lot" DROP CONSTRAINT "parking_lot_owner_id_fkey";

-- DropIndex
DROP INDEX IF EXISTS "transaction_history_id_key";
DROP INDEX IF EXISTS "parking_history_transaction_id_key";
DROP INDEX IF EXISTS "parking_history_owner_id_key";
DROP INDEX IF EXISTS "parking_history_id_key";
DROP INDEX IF EXISTS "parking_lot_owner_id_key";
DROP INDEX IF EXISTS "parking_lot_id_key";
DROP INDEX IF EXISTS "user_phone_number_key";
DROP INDEX IF EXISTS "user_id_key";

-- DropTable
DROP TABLE IF EXISTS "transaction_history";
DROP TABLE IF EXISTS "parking_history";
DROP TABLE IF EXISTS "parking_lot";
DROP TABLE IF EXISTS "user";

-- DropEnum
DROP TYPE IF EXISTS "payment_type";
DROP TYPE IF EXISTS "vehicle_type";
DROP TYPE IF EXISTS "ticket_status";
DROP TYPE IF EXISTS "user_status";
DROP TYPE IF EXISTS "role";
