USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_GetScheduledCampaigns
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
        C.CreatedBy,
        C.CreatedAt,
        C.UpdatedAt
    FROM EmailCampaign C
    INNER JOIN EmailTemplate T ON C.TemplateID = T.TemplateID
    WHERE C.Status = 1
      AND C.ScheduledAt <= GETDATE();
END
GO
