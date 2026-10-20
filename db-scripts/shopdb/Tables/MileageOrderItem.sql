USE RanShop
GO

CREATE TABLE dbo.MileageOrderItem (
    OrderItemID    BIGINT          IDENTITY(1,1)   PRIMARY KEY,
    OrderID        BIGINT          NOT NULL REFERENCES dbo.MileageOrder(OrderID),
    UserNum        INT             NOT NULL,
    ProductNum     INT             NOT NULL,
    ItemName       NVARCHAR(100)   NOT NULL,
    ItemMain       INT             NOT NULL,
    ItemSub        INT             NOT NULL,
    Quantity       INT             NOT NULL,
    UnitPrice      DECIMAL(18,4)   NOT NULL,
    DiscountType   TINYINT         NULL,
    DiscountValue  DECIMAL(18,4)   NULL,
    FinalUnitPrice DECIMAL(18,4)   NOT NULL,
    LineTotal      DECIMAL(18,4)   NOT NULL
);
GO

CREATE INDEX IX_MileageOrderItem_OrderID   ON dbo.MileageOrderItem(OrderID);
GO
CREATE INDEX IX_MileageOrderItem_UserNum   ON dbo.MileageOrderItem(UserNum);
GO
CREATE INDEX IX_MileageOrderItem_ProductNum ON dbo.MileageOrderItem(ProductNum);
GO

-- DiscountType values (snapshot from YXShopItemMap at time of purchase):
-- 1 = Percentage
-- 2 = Fixed amount
