ALTER TABLE "transaction_history" DROP COLUMN "order_id",
ADD COLUMN "order_id" TEXT;  -- Assuming the original order_id column was of type TEXT