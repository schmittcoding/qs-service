USE [RanWeb]
GO

CREATE TABLE ShopPurchaseLogs (
    ShopPurchaseID      UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID(),

    PreviousBalance     MONEY NOT NULL DEFAULT 0,
    RemainingBalance    MONEY NOT NULL DEFAULT 0,

    TotalCost           MONEY NOT NULL DEFAULT 0,
    TotalQuantity       INT NOT NULL DEFAULT 0,

    UserNum             BIGINT NOT NULL DEFAULT 1,
    UserName            NVARCHAR(20) NOT NULL,

    CreatedAt           DATETIME NOT NULL DEFAULT GETDATE()
);

CREATE TABLE ShopPurchaseItemLogs (
    ShopPurchaseItemLogID   UNIQUEIDENTIFIER NOT NULL PRIMARY KEY DEFAULT NEWID(),
    ShopPurchaseID          UNIQUEIDENTIFIER NOT NULL,

    ProductNum              INT NOT NULL,
    ItemMain                INT NOT NULL,
    DispatchItemSub         INT NOT NULL,
    ItemName                NVARCHAR(255) NOT NULL,
    Quantity                INT NOT NULL,

    UnitPrice               MONEY NOT NULL,
    TotalPrice              MONEY NOT NULL,

    CONSTRAINT FK_ShopPurchaseItemLogs_Parent
        FOREIGN KEY (ShopPurchaseID)
        REFERENCES ShopPurchaseLogs (ShopPurchaseID)
)
