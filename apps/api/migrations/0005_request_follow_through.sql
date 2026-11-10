-- Account preferences affect phone delivery only. Inbox evidence is never suppressed.
CREATE TABLE notification_preferences (
 user_id uuid PRIMARY KEY REFERENCES users,
 request_alerts text NOT NULL DEFAULT 'all' CHECK(request_alerts IN ('all','urgent','off')),
 quiet_enabled boolean NOT NULL DEFAULT false,
 timezone text NOT NULL DEFAULT 'UTC',
 quiet_start smallint NOT NULL DEFAULT 1320 CHECK(quiet_start BETWEEN 0 AND 1439),
 quiet_end smallint NOT NULL DEFAULT 480 CHECK(quiet_end BETWEEN 0 AND 1439),
 updated_at timestamptz NOT NULL DEFAULT now(), CHECK(quiet_start<>quiet_end)
);
ALTER TABLE push_deliveries DROP CONSTRAINT push_deliveries_status_check;
ALTER TABLE push_deliveries ADD CONSTRAINT push_deliveries_status_check
 CHECK(status IN ('pending','processing','sent','failed','expired','suppressed'));
ALTER TABLE push_deliveries ADD COLUMN expires_at timestamptz NOT NULL DEFAULT (now()+interval '24 hours');
ALTER TABLE push_deliveries ADD COLUMN lease_token uuid;

-- One policy for initial enqueue, retries, and pending jobs after a preference change.
-- PostgreSQL supplies IANA timezone/DST rules without a second timezone dependency.
CREATE FUNCTION request_phone_after(person uuid, priority text, at_time timestamptz)
 RETURNS timestamptz LANGUAGE plpgsql STABLE AS $$
DECLARE pref notification_preferences%ROWTYPE; local_time timestamp; minute_of_day int; end_date date;
BEGIN
 SELECT * INTO pref FROM notification_preferences WHERE user_id=person;
 IF NOT FOUND THEN RETURN at_time; END IF;
 IF pref.request_alerts='off' OR (pref.request_alerts='urgent' AND priority<>'urgent') THEN RETURN NULL; END IF;
 IF NOT pref.quiet_enabled THEN RETURN at_time; END IF;
 local_time := at_time AT TIME ZONE pref.timezone;
 minute_of_day := extract(hour FROM local_time)::int*60+extract(minute FROM local_time)::int;
 IF (pref.quiet_start<pref.quiet_end AND minute_of_day>=pref.quiet_start AND minute_of_day<pref.quiet_end)
 OR (pref.quiet_start>pref.quiet_end AND (minute_of_day>=pref.quiet_start OR minute_of_day<pref.quiet_end)) THEN
  end_date := local_time::date;
  IF pref.quiet_start>pref.quiet_end AND minute_of_day>=pref.quiet_start THEN end_date:=end_date+1; END IF;
  RETURN (end_date::timestamp+make_interval(mins=>pref.quiet_end::int)) AT TIME ZONE pref.timezone;
 END IF;
 RETURN at_time;
END $$;

ALTER TABLE service_requests ADD COLUMN last_reminded_at timestamptz;
CREATE TABLE request_follow_ups (
 request_id uuid PRIMARY KEY REFERENCES service_requests,
 reminder_at timestamptz, reminder_processed_at timestamptz,
 escalation_at timestamptz, escalation_user_id uuid REFERENCES users, escalation_processed_at timestamptz,
 updated_at timestamptz NOT NULL DEFAULT now(),
 CHECK((escalation_at IS NULL)=(escalation_user_id IS NULL)),
 CHECK(reminder_at IS NULL OR escalation_at IS NULL OR escalation_at>reminder_at)
);
CREATE INDEX request_reminder_due ON request_follow_ups(reminder_at)
 WHERE reminder_processed_at IS NULL AND reminder_at IS NOT NULL;
CREATE INDEX request_escalation_due ON request_follow_ups(escalation_at)
 WHERE escalation_processed_at IS NULL AND escalation_at IS NOT NULL;
CREATE INDEX request_notification_lookup ON notifications ((payload_json->>'request_id'));
CREATE TABLE request_activity (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), request_id uuid NOT NULL REFERENCES service_requests,
 actor_user_id uuid REFERENCES users, kind text NOT NULL,
 details jsonb NOT NULL DEFAULT '{}', created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX request_activity_timeline ON request_activity(request_id,created_at,id);
CREATE TRIGGER immutable_request_activity BEFORE UPDATE OR DELETE ON request_activity
 FOR EACH ROW EXECUTE FUNCTION immutable_record();
