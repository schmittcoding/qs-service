USE RanWeb;
GO

CREATE TABLE EventMatchGimmick (
    GimmickID         UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
    MatchID           UNIQUEIDENTIFIER NOT NULL,
    GimmickType       VARCHAR(20) NOT NULL,
    GimmickStartTime  DATETIME NOT NULL,
    GimmickEndTime    DATETIME NOT NULL,
    CreatedAt         DATETIME NOT NULL DEFAULT GETDATE(),

    CONSTRAINT FK_Gimmick_Match FOREIGN KEY (MatchID)
        REFERENCES MatchSchedule(MatchID),
    CONSTRAINT CK_Gimmick_Type CHECK (GimmickType IN ('DoublePoints','NullDeaths')),
    CONSTRAINT CK_Gimmick_Window CHECK (GimmickEndTime > GimmickStartTime)
);
GO

CREATE INDEX IX_Gimmick_Match ON EventMatchGimmick(MatchID);
GO
-- Note: window-within-match rule enforced in Rust service layer (cross-table).
