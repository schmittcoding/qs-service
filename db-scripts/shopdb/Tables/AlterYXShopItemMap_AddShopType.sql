-- Added 28 May 2026, Schmitt
-- Mileage System: Add ShopType column (1=WebPoints, 2=Mileage)
ALTER TABLE RanShop.dbo.YXShopItemMap
ADD ShopType TINYINT NOT NULL DEFAULT 1
