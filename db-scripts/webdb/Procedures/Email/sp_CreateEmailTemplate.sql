USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_CreateEmailTemplate
    @TemplateName     NVARCHAR(100),
    @TemplateCategory NVARCHAR(50),
    @SubjectLine      NVARCHAR(200),
    @HtmlBody         NVARCHAR(MAX),
    @Status           TINYINT,
    @CreatedBy        NVARCHAR(50)
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @NewID UNIQUEIDENTIFIER = NEWID();

    INSERT INTO EmailTemplate (TemplateID, TemplateName, TemplateCategory, SubjectLine, HtmlBody, Status, CreatedBy)
    VALUES (@NewID, @TemplateName, @TemplateCategory, @SubjectLine, @HtmlBody, @Status, @CreatedBy);

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
    WHERE TemplateID = @NewID;
END
GO
