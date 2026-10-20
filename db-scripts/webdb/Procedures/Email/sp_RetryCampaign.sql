USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_RetryCampaign
    @CampaignId UNIQUEIDENTIFIER
AS
BEGIN
    SET NOCOUNT ON;

    UPDATE CampaignRecipient
    SET SendStatus    = 0,
        AttemptCount  = 2,
        ErrorMessage  = NULL
    WHERE CampaignID = @CampaignId
      AND SendStatus = 2;

    DECLARE @ResetCount INT = @@ROWCOUNT;

    UPDATE EmailCampaign
    SET Status      = 2,
        TotalFailed = 0,
        UpdatedAt   = GETDATE()
    WHERE CampaignID = @CampaignId
      AND Status = 5;

    SELECT @ResetCount AS reset_count;
END
GO