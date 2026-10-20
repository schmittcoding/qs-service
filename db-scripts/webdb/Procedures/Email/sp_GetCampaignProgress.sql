USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_GetCampaignProgress
    @CampaignID UNIQUEIDENTIFIER
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        C.CampaignID,
        C.Status,
        C.TotalRecipients,
        C.TotalSent,
        C.TotalFailed,
        ISNULL((SELECT COUNT(*) FROM CampaignRecipient R WHERE R.CampaignID = C.CampaignID AND R.SendStatus = 0), 0) AS TotalPending
    FROM EmailCampaign C
    WHERE C.CampaignID = @CampaignID;
END
GO
