USE RanGameS1;
GO

CREATE OR ALTER TRIGGER trg_update_levelup_date
ON ChaInfo
AFTER UPDATE
AS
BEGIN
    SET NOCOUNT ON;

    IF UPDATE(ChaLevel)
    BEGIN
        UPDATE c
        SET LevelUpDate = GETDATE()
        FROM ChaInfo c
        INNER JOIN inserted i ON c.ChaNum = i.ChaNum
        INNER JOIN deleted d ON d.ChaNum = c.ChaNum
        WHERE i.ChaLevel <> d.ChaLevel;
    END
END;
