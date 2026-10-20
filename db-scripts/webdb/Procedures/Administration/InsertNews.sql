USE [RanWeb];
GO

CREATE OR ALTER PROCEDURE InsertNews
    @NewsTitle NVARCHAR(255),
    @NewsSlug NVARCHAR(50),
    @NewsContent NVARCHAR(MAX),
    @NewsAuthor UNIQUEIDENTIFIER
AS
BEGIN
    SET NOCOUNT ON;
    
    INSERT INTO [dbo].[News] ([NewsTitle], [NewsSlug], [NewsContent], [NewsAuthor])
    VALUES (@NewsTitle, @NewsSlug, @NewsContent, @NewsAuthor);
    
    -- Return the ID of the inserted row
    SELECT SCOPE_IDENTITY() AS InsertedNewsID;
END;
