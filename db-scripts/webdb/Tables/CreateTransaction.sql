USE [RanWeb]
GO

CREATE TABLE GatewayInfo (
    GatewayID uniqueidentifier NOT NULL PRIMARY KEY DEFAULT NEWSEQUENTIALID(),
    GatewayProvider SMALLINT NOT NULL,
    GatewayName NVARCHAR(MAX) NOT NULL,
    GatewayNumber NVARCHAR(MAX) NOT NULL,
    GatewayImage VARBINARY(MAX) NOT NULL,
    GatewayStatus SMALLINT NOT NULL
);

CREATE TABLE RechargeInfo (
    RechargeID UNIQUEIDENTIFIER NOT NULL PRIMARY KEY DEFAULT NEWSEQUENTIALID(),
    RechargeType SMALLINT NOT NULL,
    RechargeAmount MONEY NOT NULL,
    RechargeDiscount MONEY NOT NULL DEFAULT 0.0,
    RechargePoints MONEY NOT NULL,
    RechargeStatus SMALLINT NOT NULL
);

CREATE TABLE WebTransaction (
    TransactionID uniqueidentifier NOT NULL PRIMARY KEY DEFAULT NEWSEQUENTIALID(),
    UserNum BIGINT NOT NULL,
    UserName NVARCHAR(20) NOT NULL,
    TransactionType SMALLINT NOT NULL,
    TransactionTime DATETIME NOT NULL DEFAULT GETDATE(),
    TransactionAmount MONEY NOT NULL,
    TransactionStatus SMALLINT NOT NULL DEFAULT 0,
    WebPoints MONEY NOT NULL,
    UpdatedAt DATETIME NOT NULL DEFAULT GETDATE(),
    TransactionCost MONEY NOT NULL DEFAULT 0,
    ProofImage VARBINARY(MAX) NULL,
    RefferenceCode NVARCHAR(50) NULL,
    AdminRemarks NVARCHAR(MAX) NULL
);

CREATE INDEX idx_TransactionStatus ON WebTransaction(TransactionStatus);
CREATE INDEX idx_WebTransactionType ON WebTransaction(TransactionType);
CREATE INDEX idx_WebTransactionUserName ON WebTransaction(UserName);
CREATE INDEX idx_WebTransactionUserName ON WebTransaction(UserName);
CREATE INDEX idx_UserNum ON WebUser(UserNum);
ALTER DATABASE RanWeb SET READ_COMMITTED_SNAPSHOT ON;

-- Search by ReferenceCode (exact or partial match)
CREATE NONCLUSTERED INDEX IX_WebTransaction_ReferenceCode
ON WebTransaction (RefferenceCode)
INCLUDE (UserName, TransactionType, TransactionStatus, TransactionTime, TransactionAmount);

-- Search by UserName
CREATE NONCLUSTERED INDEX IX_WebTransaction_UserName
ON WebTransaction (UserName)
INCLUDE (TransactionType, TransactionStatus, TransactionTime, TransactionAmount);

-- Filter or dashboard listing by TransactionType + TransactionStatus
CREATE NONCLUSTERED INDEX IX_WebTransaction_Type_Status
ON WebTransaction (TransactionType, TransactionStatus)
INCLUDE (TransactionTime, UserName, TransactionAmount, RefferenceCode);

-- (Optional) Recently updated or sorted transactions
CREATE NONCLUSTERED INDEX IX_WebTransaction_TransactionTime
ON WebTransaction (TransactionTime DESC);
