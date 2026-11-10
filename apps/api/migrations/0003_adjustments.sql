CREATE TABLE volunteer_adjustments (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), entry_id uuid NOT NULL REFERENCES volunteer_entries,
 actor_user_id uuid NOT NULL REFERENCES users, reason text NOT NULL CHECK(length(reason) BETWEEN 1 AND 2000),
 previous_json jsonb NOT NULL, created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TRIGGER immutable_volunteer_adjustments BEFORE UPDATE OR DELETE ON volunteer_adjustments FOR EACH ROW EXECUTE FUNCTION immutable_record();
ALTER TABLE volunteer_decision_events ADD COLUMN token_hash text;
