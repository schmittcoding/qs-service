USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_CreateEmailCampaign
    @CampaignName NVARCHAR(100),
    @TemplateID   UNIQUEIDENTIFIER,
    @SegmentType  TINYINT,
    @SegmentParam INT = NULL,
    @ScheduledAt  DATETIME = NULL,
    @CreatedBy    NVARCHAR(50)
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @NewID UNIQUEIDENTIFIER = NEWID();
    DECLARE @InitialStatus TINYINT = CASE WHEN @ScheduledAt IS NOT NULL THEN 1 ELSE 0 END;

    INSERT INTO EmailCampaign (CampaignID, CampaignName, TemplateID, SegmentType, SegmentParam, Status, ScheduledAt, CreatedBy)
    VALUES (@NewID, @CampaignName, @TemplateID, @SegmentType, @SegmentParam, @InitialStatus, @ScheduledAt, @CreatedBy);

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
    WHERE C.CampaignID = @NewID;
END
GO
