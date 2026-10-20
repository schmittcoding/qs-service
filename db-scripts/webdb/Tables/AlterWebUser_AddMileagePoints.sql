-- Added 28 May 2026, Schmitt
-- Mileage System: Add MileagePoints column to WebUser
ALTER TABLE RanWeb.dbo.WebUser
ADD MileagePoints DECIMAL(18,4) NOT NULL DEFAULT 0
