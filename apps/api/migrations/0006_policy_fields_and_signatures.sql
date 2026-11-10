ALTER TABLE term_versions ADD COLUMN form_schema_json jsonb NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE term_signatures ADD COLUMN answers_json jsonb NOT NULL DEFAULT '{}'::jsonb;
ALTER TABLE term_signatures ADD COLUMN signature_svg text;
