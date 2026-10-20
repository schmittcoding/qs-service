USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_GetEmailTemplateById
    @TemplateID UNIQUEIDENTIFIER
AS
BEGIN
    SET NOCOUNT ON;

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
