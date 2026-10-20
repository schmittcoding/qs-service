USE [RanShop]
GO

ALTER TABLE ShopPurchase ADD Remarks NVARCHAR(MAX) NULL;

CREATE TABLE ItemDispatchLog (
    DispatchID                  UNIQUEIDENTIFIER NOT NULL PRIMARY KEY DEFAULT NEWID(),
    DispatchBy                  UNIQUEIDENTIFIER NOT NULL,
    DispatchNotes               NVARCHAR(255) NULL,
    CreatedAt                   DATETIME DEFAULT GETDATE()
)

CREATE NONCLUSTERED INDEX IX_ItemDispatchLog_DispatchBy ON ItemDispatchLog (DispatchBy, CreatedAt)
INCLUDE (DispatchNotes);

CREATE TABLE ItemDispatchItemLog (
    DispatchItemID              UNIQUEIDENTIFIER NOT NULL PRIMARY KEY DEFAULT NEWID(),
    DispatchID                  UNIQUEIDENTIFIER NOT NULL,
    DispatchProductNum          INT NOT NULL,
    DispatchItemMain            INT NOT NULL,
    DispatchItemSub             INT NOT NULL,
    DispatchItemName            NVARCHAR(255) NOT NULL,
    CreatedAt                   DATETIME DEFAULT GETDATE()
)

CREATE NONCLUSTERED INDEX IX_ItemDispatchItemLog_ProductNum ON ItemDispatchItemLog (DispatchProductNum)
INCLUDE (DispatchID, DispatchItemName, CreatedAt);

CREATE NONCLUSTERED INDEX IX_ItemDispatchItemLog_ItemName ON ItemDispatchItemLog (DispatchItemName)
INCLUDE (DispatchID, DispatchProductNum, CreatedAt);

CREATE TABLE ItemDispatchChaLog (
    DispatchChaID               UNIQUEIDENTIFIER NOT NULL PRIMARY KEY DEFAULT NEWID(),
    DispatchID                  UNIQUEIDENTIFIER NOT NULL,
    DispatchUserNum             BIGINT NOT NULL, 
    DispatchChaNum              INT NOT NULL,
    DispatchChaName             NVARCHAR(33) NOT NULL,
    CreatedAt                   DATETIME DEFAULT GETDATE()
)

CREATE NONCLUSTERED INDEX IX_ItemDispatchChaLog_DispatchUserNum ON ItemDispatchChaLog (DispatchUserNum)
INCLUDE (DispatchID, DispatchChaNum, DispatchChaName, CreatedAt);

CREATE NONCLUSTERED INDEX IX_ItemDispatchChaLog_DispatchChaNum ON ItemDispatchChaLog (DispatchChaNum)
INCLUDE (DispatchID, DispatchUserNum, DispatchChaName, CreatedAt);

CREATE NONCLUSTERED INDEX IX_ItemDispatchChaLog_DispatchChaName ON ItemDispatchChaLog (DispatchChaName)
INCLUDE (DispatchID, DispatchUserNum, DispatchChaNum, CreatedAt);
