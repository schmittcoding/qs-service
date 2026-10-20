USE RanWeb;
GO

IF NOT EXISTS (SELECT * FROM sys.tables WHERE name = 'EmailTemplate')
BEGIN
    CREATE TABLE EmailTemplate (
        TemplateID       UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
        TemplateName     NVARCHAR(100) NOT NULL,
        TemplateCategory NVARCHAR(50) NOT NULL DEFAULT 'general',
        SubjectLine      NVARCHAR(200) NOT NULL,
        HtmlBody         NVARCHAR(MAX) NOT NULL,
        Status           TINYINT NOT NULL DEFAULT 0,   -- 0=Draft, 1=Active, 2=Inactive
        Version          INT NOT NULL DEFAULT 1,
        CreatedBy        NVARCHAR(50) NOT NULL,
        CreatedAt        DATETIME NOT NULL DEFAULT GETDATE(),
        UpdatedAt        DATETIME NOT NULL DEFAULT GETDATE()
    );

    CREATE INDEX IX_EmailTemplate_Status ON EmailTemplate(Status);
    CREATE INDEX IX_EmailTemplate_Category ON EmailTemplate(TemplateCategory);
END
GO
