USE RanWeb;
GO

-- Returns the first campaign in Queued (2) or Sending (3) status for the worker to process.
CREATE OR ALTER PROCEDURE sp_GetActiveCampaign
AS
BEGIN
    SET NOCOUNT ON;

    SELECT TOP 1
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
    WHERE C.Status IN (2, 3)
    ORDER BY C.CreatedAt ASC;
END
GO
