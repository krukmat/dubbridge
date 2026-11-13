-- CANDIDATE ONLY, DO NOT EXECUTE AUTOMATICALLY.
-- Explicit owner/operator approval and snapshot required for destructive rollback.
-- Not in infra/migrations. Dropping this table destroys dispatch-intent evidence.
DROP TABLE fenix_dispatch_intents;
DROP FUNCTION reject_unsafe_fenix_dispatch_intent_mutation();
