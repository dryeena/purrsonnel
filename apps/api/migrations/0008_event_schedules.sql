CREATE TABLE event_shifts (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), event_edition_id uuid NOT NULL REFERENCES event_editions,
 department_id uuid NOT NULL REFERENCES departments, title text NOT NULL CHECK(length(title) BETWEEN 1 AND 160),
 starts_at timestamptz NOT NULL, ends_at timestamptz NOT NULL, location_label text NOT NULL DEFAULT '',
 slots int NOT NULL CHECK(slots BETWEEN 1 AND 200), created_by_user_id uuid NOT NULL REFERENCES users,
 created_at timestamptz NOT NULL DEFAULT now(), CHECK(ends_at>starts_at)
);
CREATE INDEX event_shifts_event_time ON event_shifts(event_edition_id,starts_at);
CREATE TABLE event_shift_assignments (
 shift_id uuid NOT NULL REFERENCES event_shifts ON DELETE CASCADE, user_id uuid NOT NULL REFERENCES users,
 assigned_by_user_id uuid REFERENCES users, claimed_at timestamptz NOT NULL DEFAULT now(), PRIMARY KEY(shift_id,user_id)
);
