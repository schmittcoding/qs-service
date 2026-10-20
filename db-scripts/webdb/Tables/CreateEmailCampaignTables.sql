USE RanWeb;
GO

IF NOT EXISTS (SELECT * FROM sys.tables WHERE name = 'EmailCampaign')
BEGIN
    CREATE TABLE EmailCampaign (
        CampaignID      UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
        CampaignName    NVARCHAR(100) NOT NULL,
        TemplateID      UNIQUEIDENTIFIER NOT NULL,
        SegmentType     TINYINT NOT NULL,            -- 0=AllPlayers, 1=VerifiedOnly, 2=OnlineWithinDays
        SegmentParam    INT NULL,                    -- days for SegmentType=2
        Status          TINYINT NOT NULL DEFAULT 0,  -- 0=Draft,1=Scheduled,2=Queued,3=Sending,4=Completed,5=Failed,6=Cancelled
        ScheduledAt     DATETIME NULL,
        StartedAt       DATETIME NULL,
        CompletedAt     DATETIME NULL,
        TotalRecipients INT NOT NULL DEFAULT 0,
        TotalSent       INT NOT NULL DEFAULT 0,
        TotalFailed     INT NOT NULL DEFAULT 0,
        CreatedBy       NVARCHAR(50) NOT NULL,
        CreatedAt       DATETIME NOT NULL DEFAULT GETDATE(),
        UpdatedAt       DATETIME NOT NULL DEFAULT GETDATE(),
        CONSTRAINT FK_Campaign_Template FOREIGN KEY (TemplateID) REFERENCES EmailTemplate(TemplateID),
        CONSTRAINT CK_Campaign_Status CHECK (Status IN (0,1,2,3,4,5,6)),
        CONSTRAINT CK_Segment_Type CHECK (SegmentType IN (0,1,2))
    );

    CREATE INDEX IX_EmailCampaign_Status ON EmailCampaign(Status);
    CREATE INDEX IX_EmailCampaign_Scheduled ON EmailCampaign(Status, ScheduledAt)
        WHERE Status = 1;
END
GO

IF NOT EXISTS (SELECT * FROM sys.tables WHERE name = 'CampaignRecipient')
BEGIN
    CREATE TABLE CampaignRecipient (
        CampaignID   UNIQUEIDENTIFIER NOT NULL,
        UserNum      BIGINT NOT NULL,
        UserEmail    NVARCHAR(100) NOT NULL,
        UserName     NVARCHAR(20) NOT NULL,
        SendStatus   TINYINT NOT NULL DEFAULT 0,  -- 0=Pending, 1=Sent, 2=Failed
        AttemptCount TINYINT NOT NULL DEFAULT 0,
        SentAt       DATETIME NULL,
        ErrorMessage NVARCHAR(500) NULL,
        PRIMARY KEY (CampaignID, UserNum),
        CONSTRAINT FK_Recipient_Campaign FOREIGN KEY (CampaignID) REFERENCES EmailCampaign(CampaignID)
    );

    CREATE INDEX IX_CampaignRecipient_Pending ON CampaignRecipient(CampaignID, SendStatus)
        WHERE SendStatus = 0;
END
GO
