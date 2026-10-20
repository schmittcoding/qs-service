USE RanShop
GO

CREATE TABLE dbo.MileageOrder (
    OrderID        BIGINT          IDENTITY(1,1)   PRIMARY KEY,
    UserNum        INT             NOT NULL,
    UserUID        NVARCHAR(50)    NOT NULL,
    OrderDate      DATETIME        NOT NULL DEFAULT GETDATE(),
    TotalItems     INT             NOT NULL,
    TotalQuantity  INT             NOT NULL,
    TotalAmount    DECIMAL(18,4)   NOT NULL,
    TotalDiscount  DECIMAL(18,4)   NOT NULL DEFAULT 0,
    PointsBefore   DECIMAL(18,4)   NOT NULL,
    PointsAfter    DECIMAL(18,4)   NOT NULL,
    OrderStatus    TINYINT         NOT NULL DEFAULT 1,
    CreatedAt      DATETIME        NOT NULL DEFAULT GETDATE()
);
GO

CREATE INDEX IX_MileageOrder_UserNum ON dbo.MileageOrder(UserNum);
GO

-- PointsBefore/PointsAfter refer to MileagePoints (not WebPoints).
-- OrderStatus values:
-- 1 = Completed
-- 2 = PartialRefund
-- 3 = FullRefund
