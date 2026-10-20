USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_GetEmailCampaignById
    @CampaignID UNIQUEIDENTIFIER
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        C.CampaignID,
        C.CampaignName,
        C.TemplateID,
        T.TemplateName,
        T.SubjectLine,
        T.HtmlBody,
        C.SegmentType,
        C.SegmentParam,
        C.Status,
        C.ScheduledAt,
        C.StartedAt,
        C.CompletedAt,
        C.TotalRecipients,
        C.TotalSent,
        C.TotalFailed,
        AI.UserName AS CreatedBy,
        C.CreatedAt,
        C.UpdatedAt
    FROM EmailCampaign C
        INNER JOIN EmailTemplate T ON C.TemplateID = T.TemplateID
        INNER JOIN AdminInfo AI ON C.CreatedBy = AI.UserID
    WHERE C.CampaignID = @CampaignID;
END
GO
