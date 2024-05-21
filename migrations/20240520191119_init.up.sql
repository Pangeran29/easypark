-- CreateEnum
CREATE TYPE "role" AS ENUM ('easypark', 'park_keeper', 'park_owner');

-- CreateEnum
CREATE TYPE "user_status" AS ENUM ('active', 'not_active');

-- CreateEnum
CREATE TYPE "ticket_status" AS ENUM ('active', 'not_active');

-- CreateEnum
CREATE TYPE "vehicle_type" AS ENUM ('car', 'motor');

-- CreateEnum
CREATE TYPE "payment_type" AS ENUM ('cash', 'qr');

-- CreateTable
CREATE TABLE "user" (
    "id" UUID NOT NULL,
    "phone_number" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "nik" TEXT NOT NULL,
    "role" "role" NOT NULL,
    "status" "user_status" NOT NULL,
    "otp" INTEGER,
    "created_at" TIMESTAMP(3) DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3)
);

-- CreateTable
CREATE TABLE "parking_lot" (
    "id" UUID NOT NULL,
    "area_name" TEXT NOT NULL,
    "address" TEXT NOT NULL,
    "image_url" TEXT NOT NULL,
    "car_cost" DOUBLE PRECISION NOT NULL,
    "motor_cost" DOUBLE PRECISION NOT NULL,
    "owner_id" UUID NOT NULL,
    "created_at" TIMESTAMP(3) DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3)
);

-- CreateTable
CREATE TABLE "parking_history" (
    "id" UUID NOT NULL,
    "ticket_status" "ticket_status" NOT NULL,
    "vehicle_type" "vehicle_type" NOT NULL,
    "payment" "payment_type" NOT NULL,
    "amount" DOUBLE PRECISION NOT NULL,
    "parking_lot_id" UUID NOT NULL,
    "easypark_id" UUID NOT NULL,
    "keeper_id" UUID NOT NULL,
    "owner_id" UUID NOT NULL,
    "transaction_id" UUID NOT NULL,
    "created_at" TIMESTAMP(3) DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3)
);

-- CreateTable
CREATE TABLE "transaction_history" (
    "id" UUID NOT NULL,
    "order_id" TEXT,
    "transaction_time" TEXT,
    "transaction_status" TEXT,
    "transaction_id" TEXT,
    "status_message" TEXT,
    "status_code" TEXT,
    "signature_key" TEXT,
    "settlement_time" TEXT,
    "payment_type" TEXT,
    "merchant_id" TEXT,
    "gross_amount" TEXT,
    "fraud_status" TEXT,
    "currency" TEXT
);

-- CreateIndex
CREATE UNIQUE INDEX "user_id_key" ON "user"("id");

-- CreateIndex
CREATE UNIQUE INDEX "user_phone_number_key" ON "user"("phone_number");

-- CreateIndex
CREATE UNIQUE INDEX "parking_lot_id_key" ON "parking_lot"("id");

-- CreateIndex
CREATE UNIQUE INDEX "parking_lot_owner_id_key" ON "parking_lot"("owner_id");

-- CreateIndex
CREATE UNIQUE INDEX "parking_history_id_key" ON "parking_history"("id");

-- CreateIndex
CREATE UNIQUE INDEX "parking_history_owner_id_key" ON "parking_history"("owner_id");

-- CreateIndex
CREATE UNIQUE INDEX "parking_history_transaction_id_key" ON "parking_history"("transaction_id");

-- CreateIndex
CREATE UNIQUE INDEX "transaction_history_id_key" ON "transaction_history"("id");

-- AddForeignKey
ALTER TABLE "parking_lot" ADD CONSTRAINT "parking_lot_owner_id_fkey" FOREIGN KEY ("owner_id") REFERENCES "user"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "parking_history" ADD CONSTRAINT "parking_history_parking_lot_id_fkey" FOREIGN KEY ("parking_lot_id") REFERENCES "parking_lot"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "parking_history" ADD CONSTRAINT "parking_history_easypark_id_fkey" FOREIGN KEY ("easypark_id") REFERENCES "user"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "parking_history" ADD CONSTRAINT "parking_history_keeper_id_fkey" FOREIGN KEY ("keeper_id") REFERENCES "user"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "parking_history" ADD CONSTRAINT "parking_history_owner_id_fkey" FOREIGN KEY ("owner_id") REFERENCES "user"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "parking_history" ADD CONSTRAINT "parking_history_transaction_id_fkey" FOREIGN KEY ("transaction_id") REFERENCES "transaction_history"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
