USE [RanWeb]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

-- Returns a row-level breakdown of valid and invalid kill log entries for a given
-- match window. Intended for cross-checking against the final tally produced by
-- sp_ExecuteTally. Each row is one deduplicated LogAction entry with attacker and
-- victim names resolved from RanGameS1.dbo.ChaInfo.
--
-- "Valid" = both attacker and victim are registered (Status=1) for the event.
-- "Invalid" = at least one side is missing or not registered.

CREATE OR ALTER PROCEDURE [dbo].[sp_GetTallyKillBreakdown]
    @EventID   UNIQUEIDENTIFIER,
    @StartTime DATETIME,
    @EndTime   DATETIME
AS
BEGIN
    SET NOCOUNT ON;

    -- Deduplicate LogAction the same way sp_ExecuteTally does:
    -- one row per (ChaNum, TargetNum, ActionDate) using MIN(ActionNum).
    SELECT
        MIN(L.ActionNum) AS ActionNum,
        L.ChaNum         AS AttackerChaNum,
        L.TargetNum      AS VictimChaNum,
        L.ActionDate
    INTO #Deduped
    FROM [RanLogS1].[dbo].[LogAction] L WITH (NOLOCK)
    WHERE L.ActionDate    >= @StartTime
      AND L.ActionDate    <  @EndTime
      AND L.baseMainMapID =  222
      AND L.baseSubMapID  =  0
      AND L.Type          =  2
    GROUP BY L.ChaNum, L.TargetNum, L.ActionDate;

    -- Resolve character names for all unique ChaNum values in scope.
    SELECT DISTINCT
        c.ChaNum,
        c.ChaName
    INTO #Names
    FROM [RanGameS1].[dbo].[ChaInfo] c
    WHERE c.ChaNum IN (
        SELECT AttackerChaNum FROM #Deduped
        UNION
        SELECT VictimChaNum   FROM #Deduped
    );

    -- Final result: one row per deduplicated kill, flagged as Valid or Invalid.
    SELECT
        d.ActionDate,
        d.AttackerChaNum,
        atk.ChaName                              AS AttackerChaName,
        d.VictimChaNum,
        vic.ChaName                              AS VictimChaName,
        CASE
            WHEN ra.ChaNum IS NOT NULL
             AND rv.ChaNum IS NOT NULL THEN 'Valid'
            ELSE 'Invalid'
        END                                      AS KillStatus,
        CASE WHEN ra.ChaNum IS NULL THEN 'Not registered' ELSE NULL END AS AttackerIssue,
        CASE WHEN rv.ChaNum IS NULL THEN 'Not registered' ELSE NULL END AS VictimIssue
    FROM #Deduped d
    LEFT JOIN #Names atk ON atk.ChaNum = d.AttackerChaNum
    LEFT JOIN #Names vic ON vic.ChaNum = d.VictimChaNum
    LEFT JOIN EventRegistration ra
        ON  ra.EventID = @EventID
        AND ra.ChaNum  = d.AttackerChaNum
        AND ra.Status  = 1
    LEFT JOIN EventRegistration rv
        ON  rv.EventID = @EventID
        AND rv.ChaNum  = d.VictimChaNum
        AND rv.Status  = 1
    ORDER BY d.ActionDate, d.AttackerChaNum, d.VictimChaNum;

    DROP TABLE #Deduped, #Names;
END
GO
