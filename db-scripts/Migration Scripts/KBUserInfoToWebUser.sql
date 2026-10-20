WITH SourceCTE AS (
    SELECT 
        [UserNum],
        [UserType],
        [UserName],
        [UserAvailable],
        [UserEmail],
        [KBPoint],
        COALESCE([CompleteName], 'DidNotComply') AS CompleteName,
        [UserPin],
        ROW_NUMBER() OVER (PARTITION BY [UserEmail] ORDER BY [UserNum]) AS rn
    FROM [KBDB].[dbo].[KBUserInfo]
)
INSERT INTO [RanWeb].[dbo].[WebUser] 
    ([UserNum], [UserType], [UserName], [UserAvailable], [UserEmail], 
     [WebPoints], [UserVerified], [UserVerificationToken], [CreatedAt], [UpdatedAt], [UserFullName], [WebPin])
SELECT 
    s.[UserNum],
    s.[UserType],
    s.[UserName],
    s.[UserAvailable],
	s.[UserEmail],
    s.[KBPoint] AS WebPoints,
    1 AS UserVerified,
    NULL AS UserVerificationToken,
    GETDATE() AS CreatedAt,
    GETDATE() AS UpdatedAt,
	s.CompleteName AS UserFullName,
	LEFT(s.UserPin, 6) AS WebPin
FROM SourceCTE s;