USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_UpdateCampaignStatus
    @CampaignID      UNIQUEIDENTIFIER,
    @NewStatus       TINYINT,
    @TotalRecipients INT = NULL,
    @TotalSent       INT = NULL,
    @TotalFailed     INT = NULL
AS
BEGIN
    SET NOCOUNT ON;

    IF NOT EXISTS (SELECT 1 FROM EmailCampaign WHERE CampaignID = @CampaignID)
    BEGIN
        SELECT 'NOT_FOUND' AS ErrorCode;
        RETURN;
    END

    UPDATE EmailCampaign
    SET
        Status          = @NewStatus,
        StartedAt       = CASE WHEN @NewStatus = 3 AND StartedAt IS NULL THEN GETDATE() ELSE StartedAt END,
        CompletedAt     = CASE WHEN @NewStatus IN (4, 5, 6) THEN GETDATE() ELSE CompletedAt END,
        TotalRecipients = ISNULL(@TotalRecipients, TotalRecipients),
        TotalSent       = ISNULL(@TotalSent, TotalSent),
        TotalFailed     = ISNULL(@TotalFailed, TotalFailed),
        UpdatedAt       = GETDATE()
    WHERE CampaignID = @CampaignID;
END
GO
