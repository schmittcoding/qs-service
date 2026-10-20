USE RanShop
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetMileageConfig]
AS
BEGIN
    SET NOCOUNT ON;

    SELECT TOP 1
        ConfigID,
        MileageRate,
        IsActive,
        EffectiveFrom,
        EffectiveTo,
        Description,
        UpdatedBy,
        CreatedAt,
        UpdatedAt
    FROM dbo.MileageConfig
    WHERE IsActive = 1
      AND (EffectiveTo IS NULL OR EffectiveTo > GETDATE())
    ORDER BY EffectiveFrom DESC;
END
GO
