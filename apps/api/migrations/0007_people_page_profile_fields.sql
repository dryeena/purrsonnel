ALTER TABLE profile_field_definitions ADD COLUMN show_on_people_page boolean NOT NULL DEFAULT false;
ALTER TABLE profile_field_definitions ADD CONSTRAINT people_page_fields_are_personnel_visible CHECK (NOT show_on_people_page OR visibility='personnel');
CREATE INDEX profile_field_values_people_page_search ON profile_field_values(definition_id,user_id);
CREATE OR REPLACE VIEW people_directory AS SELECT u.id,u.myfwa_id,u.display_name,u.status,u.primary_department_id,u.personnel_level_id,u.volunteer_eligible,u.created_at,u.updated_at,l.label AS level,l.rank,d.name AS department,
 COALESCE((SELECT jsonb_agg(jsonb_build_object('id',dd.id,'name',dd.name)) FROM user_departments m JOIN departments dd ON dd.id=m.department_id WHERE m.user_id=u.id AND m.ended_at IS NULL),'[]') AS departments,
 COALESCE((SELECT jsonb_agg(year ORDER BY year) FROM user_service_years y WHERE y.user_id=u.id AND verification_status='verified'),'[]') AS service_years,
 (SELECT count(*) FROM user_service_years y WHERE y.user_id=u.id AND verification_status='verified') AS verified_years,
 COALESCE((SELECT jsonb_agg(s.name) FROM user_skills us JOIN skills s ON s.id=us.skill_id WHERE us.user_id=u.id),'[]') AS skills,
 COALESCE((SELECT jsonb_object_agg(f.key,pfv.value_json) FROM profile_field_definitions f JOIN profile_field_values pfv ON pfv.definition_id=f.id WHERE pfv.user_id=u.id AND f.active AND f.show_on_people_page),'{}') AS profile_fields
 FROM users u LEFT JOIN personnel_levels l ON l.id=u.personnel_level_id LEFT JOIN departments d ON d.id=u.primary_department_id;
