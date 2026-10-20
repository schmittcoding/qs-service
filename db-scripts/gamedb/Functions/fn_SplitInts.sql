USE RanGameS1
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER FUNCTION [dbo].[fn_SplitInts]
(
    @Input NVARCHAR(MAX),
    @Delimiter CHAR(1)
)
RETURNS @Result TABLE (Value INT)
AS
BEGIN
    DECLARE @Index INT
    DECLARE @Slice NVARCHAR(4000)

    WHILE LEN(@Input) > 0
    BEGIN
        SET @Index = CHARINDEX(@Delimiter, @Input)

        IF @Index > 0
        BEGIN
            SET @Slice = LEFT(@Input, @Index - 1)
            INSERT INTO @Result (Value) 
            SELECT TRY_CAST(@Slice AS INT)
            WHERE ISNUMERIC(@Slice) = 1
            SET @Input = RIGHT(@Input, LEN(@Input) - @Index)
        END
        ELSE
        BEGIN
            INSERT INTO @Result (Value) 
            SELECT TRY_CAST(@Input AS INT)
            WHERE ISNUMERIC(@Input) = 1
            SET @Input = ''
        END
    END

    RETURN
END