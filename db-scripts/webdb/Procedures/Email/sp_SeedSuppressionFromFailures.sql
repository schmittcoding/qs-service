USE RanWeb;
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_SeedSuppressionFromFailures]
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @InsertedCount INT = 0;

    BEGIN TRANSACTION;
    BEGIN TRY
        -- Insert from EmailSentLog where Status = 'failed'
        INSERT INTO [dbo].[EmailSuppression] (Email, Reason, Source)
        SELECT DISTINCT
            LOWER(RecipientEmail),
            'bounce',
            'seed'
        FROM [dbo].[EmailSentLog]
        WHERE Status = 'failed'
            AND LOWER(RecipientEmail) NOT IN (
                SELECT Email FROM [dbo].[EmailSuppression] WHERE RemovedAt IS NULL
            );

        SET @InsertedCount = @@ROWCOUNT;

        -- Insert from CampaignRecipient where SendStatus = 2 (failed) and AttemptCount >= 3
        IF EXISTS (SELECT 1 FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'CampaignRecipient' AND TABLE_SCHEMA = 'dbo')
        BEGIN
            INSERT INTO [dbo].[EmailSuppression] (Email, Reason, Source)
            SELECT DISTINCT
                LOWER(R.UserEmail),
                'bounce',
                'seed'
            FROM [dbo].[CampaignRecipient] R
            WHERE R.SendStatus = 2 AND R.AttemptCount >= 3
                AND LOWER(R.UserEmail) NOT IN (
                    SELECT Email FROM [dbo].[EmailSuppression] WHERE RemovedAt IS NULL
                );

            SET @InsertedCount = @InsertedCount + @@ROWCOUNT;
        END

        COMMIT TRANSACTION;

        SELECT @InsertedCount AS NewEntriesInserted;
    END TRY
    BEGIN CATCH
        ROLLBACK TRANSACTION;
        THROW;
    END CATCH
END
GO
