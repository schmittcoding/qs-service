USE RanWeb;
GO

-- Add EventSlug column (nullable first so existing rows don't violate NOT NULL)
ALTER TABLE EventDefinition ADD EventSlug VARCHAR(100) NULL;
GO

-- Backfill: derive slug from EventName (lowercase, spaces -> hyphens, strip non [a-z0-9-])
-- Developer: review for collisions before running on production.
-- Example generation logic (adjust if EventName contains non-ASCII):
UPDATE EventDefinition
SET EventSlug = LOWER(
    REPLACE(
        REPLACE(
            REPLACE(EventName COLLATE Latin1_General_CI_AI, ' ', '-'),
        '.', ''),
    '_', '-')
)
WHERE EventSlug IS NULL OR EventSlug = '';
GO

-- After verifying no collisions exist, enforce NOT NULL and UNIQUE
ALTER TABLE EventDefinition ALTER COLUMN EventSlug VARCHAR(100) NOT NULL;
GO

ALTER TABLE EventDefinition
ADD CONSTRAINT UQ_EventDefinition_EventSlug UNIQUE (EventSlug);
GO
