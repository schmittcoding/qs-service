
CREATE NONCLUSTERED INDEX IX_ShopPurchase_PurFlag ON RanShop.dbo.ShopPurchase (PurFlag)
INCLUDE (UserUID, ProductNum, PurDate);

CREATE INDEX IX_ShopPurchase_PurDate
ON dbo.ShopPurchase (PurDate DESC)
INCLUDE (ProductNum, UserUID, PurFlag, PurChgDate, ItemName);
