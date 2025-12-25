-- Add invite activity types to the CHECK constraint

-- Drop the existing CHECK constraint
ALTER TABLE organization_activities DROP CONSTRAINT IF EXISTS organization_activities_activity_type_check;

-- Add new CHECK constraint with invite activity types
ALTER TABLE organization_activities ADD CONSTRAINT organization_activities_activity_type_check
    CHECK (activity_type IN (
        'org_created',
        'member_added',
        'member_removed',
        'member_role_changed',
        'org_name_changed',
        'invite_sent',
        'invite_accepted',
        'invite_declined'
    ));
