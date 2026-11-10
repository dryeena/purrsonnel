CREATE TABLE service_requests (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), sender_user_id uuid NOT NULL REFERENCES users,
 recipient_user_id uuid REFERENCES users, recipient_department_id uuid REFERENCES departments,
 title text NOT NULL CHECK(length(title) BETWEEN 1 AND 160), body text NOT NULL CHECK(length(body) BETWEEN 1 AND 4000),
 urgency text NOT NULL CHECK(urgency IN ('normal','urgent')), due_at timestamptz,
 status text NOT NULL DEFAULT 'open' CHECK(status IN ('open','accepted','completed','cancelled')),
 assigned_to_user_id uuid REFERENCES users, idempotency_key uuid NOT NULL, payload_sha256 text NOT NULL,
 created_at timestamptz NOT NULL DEFAULT now(), updated_at timestamptz NOT NULL DEFAULT now(), completed_at timestamptz,
 CHECK((recipient_user_id IS NULL)<>(recipient_department_id IS NULL)), UNIQUE(sender_user_id,idempotency_key)
);
CREATE TABLE request_recipients (
 request_id uuid REFERENCES service_requests, user_id uuid REFERENCES users, read_at timestamptz,
 PRIMARY KEY(request_id,user_id)
);
CREATE INDEX request_recipient_inbox ON request_recipients(user_id,request_id);
CREATE TABLE push_subscriptions (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), user_id uuid NOT NULL REFERENCES users,
 endpoint_hash text UNIQUE NOT NULL, endpoint text NOT NULL, p256dh text NOT NULL, auth text NOT NULL,
 active boolean NOT NULL DEFAULT true, created_at timestamptz NOT NULL DEFAULT now(), updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE push_deliveries (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), notification_id uuid NOT NULL REFERENCES notifications,
 subscription_id uuid NOT NULL REFERENCES push_subscriptions,
 status text NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','processing','sent','failed','expired')),
 attempts int NOT NULL DEFAULT 0, next_attempt_at timestamptz NOT NULL DEFAULT now(), leased_until timestamptz,
 last_error text, sent_at timestamptz, created_at timestamptz NOT NULL DEFAULT now(),
 UNIQUE(notification_id,subscription_id)
);
CREATE INDEX push_delivery_queue ON push_deliveries(status,next_attempt_at);

-- Explicit capability grants for fictional development identities only. New real users
-- never receive request-sending authority merely through their personnel level.
INSERT INTO permission_grants(user_id,permission,scope_type)
 SELECT id,'requests.send','global' FROM users WHERE email='dev.hr@purrsonnel.local' AND dev_identity;
INSERT INTO permission_grants(user_id,permission,scope_type,scope_id)
 SELECT DISTINCT u.id,'requests.send','department',m.department_id FROM users u
 JOIN user_departments m ON m.user_id=u.id AND m.ended_at IS NULL
 WHERE u.dev_identity AND u.status='active' AND u.email<>'dev.hr@purrsonnel.local';
