USE RanWeb;
GO

CREATE OR ALTER PROCEDURE sp_GetEmailTemplates
    @Status   TINYINT        = NULL,
    @Category NVARCHAR(50)   = NULL,
    @Search   NVARCHAR(100)  = NULL,
    @Page     INT            = 1,
    @PageSize INT            = 10
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    -- Result set 1: total count
    SELECT COUNT(*) AS TotalRecords
    FROM EmailTemplate
    WHERE (@Status IS NULL OR Status = @Status)
      AND (@Category IS NULL OR TemplateCategory = @Category)
      AND (@Search IS NULL OR TemplateName LIKE '%' + @Search + '%' OR SubjectLine LIKE '%' + @Search + '%');

    -- Result set 2: paginated data
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
    WHERE (@Status IS NULL OR Status = @Status)
      AND (@Category IS NULL OR TemplateCategory = @Category)
      AND (@Search IS NULL OR TemplateName LIKE '%' + @Search + '%' OR SubjectLine LIKE '%' + @Search + '%')
    ORDER BY CreatedAt DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
