-- DropForeignKey
ALTER TABLE "parking_lot" DROP CONSTRAINT "parking_lot_keeper_id_fkey";

-- DropIndex
DROP INDEX "parking_lot_keeper_id_key";

-- AlterTable
ALTER TABLE "parking_lot" DROP COLUMN "keeper_id";
