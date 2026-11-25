CREATE TABLE permission_thresholds (
 permission text PRIMARY KEY,
 minimum_rank integer NOT NULL CHECK (minimum_rank BETWEEN 1 AND 1000)
);
CREATE OR REPLACE FUNCTION has_permission(actor uuid, capability text, dept uuid) RETURNS boolean LANGUAGE sql STABLE AS $$
 SELECT EXISTS(SELECT 1 FROM permission_grants g WHERE g.user_id=actor AND g.permission=capability AND
 (g.scope_type='global' OR (g.scope_type='department' AND g.scope_id=dept) OR
 (g.scope_type='division' AND EXISTS(SELECT 1 FROM departments d WHERE d.id=dept AND d.division_id=g.scope_id))))
 OR EXISTS(SELECT 1 FROM users u JOIN personnel_levels l ON l.id=u.personnel_level_id
 JOIN permission_thresholds t ON t.permission=capability
 WHERE u.id=actor AND u.status='active' AND l.active AND l.rank>=t.minimum_rank)
$$;
