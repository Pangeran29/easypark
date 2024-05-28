/*
  Warnings:

  - You are about to drop the column `keeper_id` on the `parking_lot` table. All the data in the column will be lost.

*/
-- DropForeignKey
ALTER TABLE "parking_lot" DROP CONSTRAINT "parking_lot_keeper_id_fkey";

-- DropIndex
DROP INDEX "parking_lot_keeper_id_key";

-- AlterTable
ALTER TABLE "parking_lot" DROP COLUMN "keeper_id";

-- AlterTable
ALTER TABLE "user" ADD COLUMN     "parking_lot_id" UUID;

-- AddForeignKey
ALTER TABLE "user" ADD CONSTRAINT "user_parking_lot_id_fkey" FOREIGN KEY ("parking_lot_id") REFERENCES "parking_lot"("id") ON DELETE SET NULL ON UPDATE CASCADE;
