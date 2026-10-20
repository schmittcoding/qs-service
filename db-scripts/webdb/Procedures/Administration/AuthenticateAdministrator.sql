CREATE PROCEDURE AuthenticateAdmin
    @UserName NVARCHAR(20),
    @UserPass NVARCHAR(255)
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @StoredHashedPass VARBINARY(64);
    DECLARE @InputHashedPass VARBINARY(64);

    -- Get the stored hashed password
    SELECT @StoredHashedPass = UserPass 
    FROM [dbo].[AdminInfo] 
    WHERE UserName = @UserName;

    -- If user not found, return failure
    IF @StoredHashedPass IS NULL
    BEGIN
        SELECT 0 AS Authenticated;
        RETURN;
    END

    -- Hash the input password
    SET @InputHashedPass = HASHBYTES('SHA2_256', @UserPass);

    -- Compare the hashed values
    IF @StoredHashedPass = @InputHashedPass
        SELECT 1 AS Authenticated;  -- Success
    ELSE
        SELECT 0 AS Authenticated;  -- Failure
END;
