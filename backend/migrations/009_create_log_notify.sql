-- Create trigger function to notify on new log inserts for SSE broadcasting
CREATE OR REPLACE FUNCTION notify_new_log()
RETURNS TRIGGER AS $$
BEGIN
    -- Notify with minimal payload (full log details fetched by listener if needed)
    PERFORM pg_notify('new_log', json_build_object(
        'project_id', NEW.project_id,
        'id', NEW.id,
        'level', NEW.level,
        'message', LEFT(NEW.message, 200),  -- Truncate message for notification
        'timestamp', NEW.timestamp,
        'source', NEW.source
    )::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger on logs table
CREATE TRIGGER logs_notify_trigger
    AFTER INSERT ON logs
    FOR EACH ROW
    EXECUTE FUNCTION notify_new_log();
