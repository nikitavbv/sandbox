update sandbox_tasks set params = jsonb_set(params, '{prompt}', to_jsonb(prompt::text), true) where params->'prompt' is null;