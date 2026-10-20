USE RanWeb;
GO

-- Populates CampaignRecipient for a given campaign based on SegmentType.
--
-- Deduplication strategy:
--   Users are allowed to register multiple accounts with the same email.
--   ROW_NUMBER() OVER (PARTITION BY LOWER(email) ORDER BY UserNum ASC)
--   selects only the oldest account per unique email address, so each
--   email address receives at most one message per campaign.
--
-- Idempotency:
--   The NOT EXISTS guard checks by email address (not UserNum), so
--   calling this SP twice for the same campaign is safe.
CREATE OR ALTER PROCEDURE sp_PopulateCampaignRecipients
    @CampaignID   UNIQUEIDENTIFIER,
    @SegmentType  TINYINT,           -- 0=AllPlayers, 1=VerifiedOnly, 2=OnlineWithinDays
    @SegmentParam INT = NULL         -- days for SegmentType=2
AS
BEGIN
    SET NOCOUNT ON;

    IF NOT EXISTS (SELECT 1 FROM EmailCampaign WHERE CampaignID = @CampaignID)
    BEGIN
        SELECT 0 AS TotalRecipients;
        RETURN;
    END

    -- Build the candidate set for the requested segment, deduplicated by email.
    -- Base filters (UserAvailable, non-empty email) apply to all segments.
    -- Segment-specific filters are inside the AND (...) block.
    ;WITH DeduplicatedCandidates AS (
        SELECT
            WU.UserNum,
            WU.UserEmail,
            WU.UserName,
            ROW_NUMBER() OVER (
                PARTITION BY LOWER(WU.UserEmail)
                ORDER BY WU.UserNum ASC      -- oldest account wins
            ) AS rn
        FROM RanWeb.dbo.WebUser WU
        WHERE WU.UserAvailable = 1
          AND WU.UserEmail IS NOT NULL
          AND LEN(WU.UserEmail) > 0
          AND (
              @SegmentType = 0              -- all active players (includes unverified)
              OR (
                  @SegmentType = 1          -- verified accounts only
                  AND WU.UserVerified = 1
              )
              OR (
                  @SegmentType = 2          -- verified + active within N days
                  AND WU.UserVerified = 1
                  AND WU.UserNum IN (
                      SELECT DISTINCT UI.UserNum
                      FROM RanUser.dbo.GSUserInfo UI
                      WHERE UI.LastLoginDate >= DATEADD(DAY, -ISNULL(@SegmentParam, 30), GETDATE())
                  )
              )
          )
    )
    INSERT INTO CampaignRecipient (CampaignID, UserNum, UserEmail, UserName)
    SELECT @CampaignID, DC.UserNum, DC.UserEmail, DC.UserName
    FROM DeduplicatedCandidates DC
    WHERE DC.rn = 1
      -- Idempotency: skip email addresses already present in this campaign
      AND NOT EXISTS (
          SELECT 1 FROM CampaignRecipient CR
          WHERE CR.CampaignID = @CampaignID
            AND LOWER(CR.UserEmail) = LOWER(DC.UserEmail)
      );

    DECLARE @RecipientCount INT = (
        SELECT COUNT(*) FROM CampaignRecipient WHERE CampaignID = @CampaignID
    );

    UPDATE EmailCampaign
    SET TotalRecipients = @RecipientCount, UpdatedAt = GETDATE()
    WHERE CampaignID = @CampaignID;

    SELECT @RecipientCount AS TotalRecipients;
END
GO
