CREATE TABLE IF NOT EXISTS "invoices" ( "id" uuid_text NOT NULL, "amount" double, "created_at" datetime_text DEFAULT CURRENT_TIMESTAMP, "updated_at" datetime_text DEFAULT CURRENT_TIMESTAMP )
