USE [RanWeb]
GO

/****** Object:  StoredProcedure [dbo].[GetNewsData]    Script Date: 2/6/2025 12:37:12 AM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

-- If the procedure exists, it will be altered; otherwise, it will be created.
CREATE OR ALTER PROCEDURE [dbo].[GetAllNews]
AS
BEGIN
    -- Select all news articles along with the username of the author
    SELECT 
        WebNews.NewsId, 
		WebNews.NewsCategory,
        WebNews.NewsTitle, 
        WebNews.NewsSlug, 
        WebNews.NewsContent,
		WebNews.NewsAuthor,
		WebNews.NewsStatus,
        WebNews.NewsCreationDate
    FROM WebNews ORDER BY WebNews.NewsCreationDate DESC;  -- Optionally order by the creation date
END;
GO
