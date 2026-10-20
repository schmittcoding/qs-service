USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_GetPendingRecipientBatch
    @CampaignID UNIQUEIDENTIFIER,
    @BatchSize  INT = 50
AS
BEGIN
    SET NOCOUNT ON;

    SELECT TOP (@BatchSize)
        CampaignID,
        UserNum,
        UserEmail,
        UserName,
        SendStatus,
        AttemptCount,
        SentAt,
        ErrorMessage
    FROM CampaignRecipient
    WHERE CampaignID = @CampaignID
      AND SendStatus = 0
      AND AttemptCount < 3
    ORDER BY UserNum ASC;
END
GO
