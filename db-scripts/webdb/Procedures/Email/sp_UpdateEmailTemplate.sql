USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_UpdateEmailTemplate
    @TemplateID       UNIQUEIDENTIFIER,
    @TemplateName     NVARCHAR(100)  = NULL,
    @TemplateCategory NVARCHAR(50)   = NULL,
    @SubjectLine      NVARCHAR(200)  = NULL,
    @HtmlBody         NVARCHAR(MAX)  = NULL,
    @Status           TINYINT        = NULL
AS
BEGIN
    SET NOCOUNT ON;

    IF NOT EXISTS (SELECT 1 FROM EmailTemplate WHERE TemplateID = @TemplateID)
    BEGIN
        SELECT 'NOT_FOUND' AS ErrorCode;
        RETURN;
    END

    UPDATE EmailTemplate
    SET
        TemplateName     = ISNULL(@TemplateName, TemplateName),
        TemplateCategory = ISNULL(@TemplateCategory, TemplateCategory),
        SubjectLine      = ISNULL(@SubjectLine, SubjectLine),
        HtmlBody         = ISNULL(@HtmlBody, HtmlBody),
        Status           = ISNULL(@Status, Status),
        Version          = Version + 1,
        UpdatedAt        = GETDATE()
    WHERE TemplateID = @TemplateID;

    SELECT
        TemplateID,
        TemplateName,
        TemplateCategory,
        SubjectLine,
        HtmlBody,
        Status,
        Version,
        CreatedBy,
        CreatedAt,
        UpdatedAt
    FROM EmailTemplate
    WHERE TemplateID = @TemplateID;
END
GO
