USE RanShop
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_UpdateMileageConfig]
    @MileageRate     DECIMAL(5,2),
    @EffectiveFrom   DATETIME        = NULL,
    @EffectiveTo     DATETIME        = NULL,
    @Description     NVARCHAR(200)   = NULL,
    @UpdatedBy       NVARCHAR(50)
AS
BEGIN
    SET NOCOUNT ON;

    BEGIN TRY
        BEGIN TRANSACTION;

        IF @EffectiveFrom IS NULL
            SET @EffectiveFrom = GETDATE();

        -- Deactivate current active config
        UPDATE dbo.MileageConfig
        SET IsActive = 0,
            UpdatedAt = GETDATE()
        WHERE IsActive = 1;

        -- Insert new config
        INSERT INTO dbo.MileageConfig (
            MileageRate, IsActive, EffectiveFrom, EffectiveTo,
            Description, UpdatedBy
        )
        VALUES (
            @MileageRate, 1, @EffectiveFrom, @EffectiveTo,
            @Description, @UpdatedBy
        );

        DECLARE @NewConfigID INT = SCOPE_IDENTITY();

        COMMIT TRANSACTION;

        SELECT
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
        WHERE ConfigID = @NewConfigID;
    END TRY
    BEGIN CATCH
        IF @@TRANCOUNT > 0
            ROLLBACK TRANSACTION;

        DECLARE @ErrorMessage NVARCHAR(4000) = ERROR_MESSAGE(),
                @ErrorSeverity INT = ERROR_SEVERITY(),
                @ErrorState INT = ERROR_STATE();
        RAISERROR (@ErrorMessage, @ErrorSeverity, @ErrorState);
    END CATCH
END
GO
