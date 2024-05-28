-- DropForeignKey
ALTER TABLE "user" DROP CONSTRAINT "user_parking_lot_id_fkey";

-- AlterTable
ALTER TABLE "user" DROP COLUMN "parking_lot_id";

-- AlterTable
ALTER TABLE "parking_lot" ADD COLUMN "keeper_id" UUID NOT NULL;

-- CreateIndex
CREATE UNIQUE INDEX "parking_lot_keeper_id_key" ON "parking_lot"("keeper_id");

-- AddForeignKey
ALTER TABLE "parking_lot" ADD CONSTRAINT "parking_lot_keeper_id_fkey" FOREIGN KEY ("keeper_id") REFERENCES "user"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
