USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_GetEmailCampaigns
    @Status   TINYINT = NULL,
    @Page     INT = 1,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    -- Result set 1: total count
    SELECT COUNT(*) AS TotalRecords
    FROM EmailCampaign
    WHERE (@Status IS NULL OR Status = @Status);

    -- Result set 2: paginated data
    SELECT
        C.CampaignID,
        C.CampaignName,
        C.TemplateID,
        T.TemplateName,
        T.SubjectLine,
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
    WHERE (@Status IS NULL OR C.Status = @Status)
    ORDER BY C.CreatedAt DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
