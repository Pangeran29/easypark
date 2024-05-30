-- AlterTable
ALTER TABLE "user" ADD COLUMN     "owner_id" UUID;

-- AddForeignKey
ALTER TABLE "user" ADD CONSTRAINT "user_owner_id_fkey" FOREIGN KEY ("owner_id") REFERENCES "user"("id") ON DELETE SET NULL ON UPDATE CASCADE;
