USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_UpdateRecipientStatus
    @CampaignID  UNIQUEIDENTIFIER,
    @UserNum     BIGINT,
    @SendStatus  TINYINT,
    @ErrorMsg    NVARCHAR(500) = NULL
AS
BEGIN
    SET NOCOUNT ON;

    UPDATE CampaignRecipient
    SET
        SendStatus   = @SendStatus,
        AttemptCount = AttemptCount + 1,
        SentAt       = CASE WHEN @SendStatus = 1 THEN GETDATE() ELSE SentAt END,
        ErrorMessage = @ErrorMsg
    WHERE CampaignID = @CampaignID AND UserNum = @UserNum;
END
GO
