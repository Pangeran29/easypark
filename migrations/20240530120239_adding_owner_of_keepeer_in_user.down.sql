-- RemoveForeignKey
ALTER TABLE "user" DROP CONSTRAINT "user_owner_id_fkey";

-- DropColumn
ALTER TABLE "user" DROP COLUMN "owner_id";
