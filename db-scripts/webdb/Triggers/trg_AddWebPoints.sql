CREATE TRIGGER trg_AddWebPoints
ON [RanWeb].[dbo].[WebTransaction]
AFTER UPDATE
AS
BEGIN
    SET NOCOUNT ON;
    BEGIN TRANSACTION;

    BEGIN TRY
        -- Check if there are any transactions that changed from not 1 to 1
        IF EXISTS (
            SELECT 1
            FROM inserted i
            INNER JOIN deleted d ON i.TransactionID = d.TransactionID
            WHERE i.TransactionStatus = 1 AND d.TransactionStatus <> 1
        )
        BEGIN
            MERGE INTO dbo.WebUser AS wu
            USING (
                SELECT i.UserNum, i.TransactionAmount 
                FROM inserted i
                INNER JOIN deleted d ON i.TransactionID = d.TransactionID
                WHERE i.TransactionStatus = 1 AND d.TransactionStatus <> 1
            ) AS src
            ON wu.UserNum = src.UserNum
            WHEN MATCHED THEN
                UPDATE SET wu.WebPoints = wu.WebPoints + src.TransactionAmount;
        END

        COMMIT TRANSACTION;
    END TRY
    BEGIN CATCH
        ROLLBACK TRANSACTION;
        THROW;
    END CATCH;
END;