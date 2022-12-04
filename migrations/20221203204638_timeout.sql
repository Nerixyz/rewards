UPDATE rewards
SET data = jsonb_set(data, '{data}', jsonb_build_object('duration', data ->> 'data', 'vip', false))
WHERE data ->> 'type' = 'Timeout'
  and jsonb_typeof(data -> 'data') = 'string';
