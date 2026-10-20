use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
#[allow(unused_imports)]
use crate::routes::{
    checks::{health_checker_handler, database_health_checker},
    auth::{
        authenticate as auth_authenticate,
        register,
        logout,
        validate,
        forgot_password,
        change_email,
        change_password,
        change_pincode,
        reset_pincode,
        validate_username,
        validate_email,
    },
    user::profile,
    news::get_news,
    admin::{auth::{authenticate as admin_authenticate}, profile::get_profile, players::{get_players, get_player_by_id, ban_player_account, ban_account_history, account_transaction_history, get_characters_by_account, get_login_history}, characters::{get_characters, get_character_by_id}, item_shop::get_shop_items, dashboard::{get_online_count, get_registered_count, get_transaction_count, get_recharge_total}},
};
use ran_models::{
    admin::{auth::{AuthAdminRequest, AuthAdminProfileData}, players::{BanAccountRequestPayload, UserAccountInfo, UserCharacterList, BanAccountHistory, UserLoginHistoryRecord}, characters::{CharacterInfo, CharacterInfoList}},
    news::NewsItem,
    users::{
        AuthRequest, ChangeEmailRequest, ChangePasswordRequest, ChangePincodeRequest,
        ForgotPasswordRequest, RegistrationRequest, ValidateEmailRequest, ValidateUsernameRequest,
    },
    events::{
        event::{CreateEventRequest, UpdateEventRequest, GenerateMatchesRequest, CreateMatchRequest, CreateMatchesRequest, UpdateMatchRequest, SetTowerResultsRequest},
        gimmick::CreateGimmickRequest,
        deduction::DeductionRequest,
        registration::{RegisterRequest, GvgRegisterRequest},
    },
    item_shop::{GetItemShopData, ItemShopData, ItemShopBase, ItemGalleryImage},
    PaginationParams,
};

/// Adds security scheme definitions to the OpenAPI spec
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);

        // Cookie-based session auth for core routes
        components.add_security_scheme(
            "cookie_session",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("session_cookie"))),
        );

        // JWT Bearer token auth for admin routes
        components.add_security_scheme(
            "bearer_jwt",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// Central OpenAPI documentation for GS API
///
/// This struct is built by the `#[derive(OpenApi)]` macro and aggregates:
/// - All route handlers via `paths![]` macro
/// - All request/response schemas via `components(schemas(...))`
/// - Security scheme definitions via the `SecurityAddon` modifier
///
/// As new routes are annotated with `#[utoipa::path(...)]`, they are registered
/// by adding them to the `paths![]` list below. Similarly, new request/response types
/// that derive `#[derive(utoipa::ToSchema)]` are registered in `components(schemas(...))`.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "GS API",
        version = "1.0.0",
        description = "RAN Online Game Server API — core (cookie session) and admin (JWT Bearer) routes"
    ),
    modifiers(&SecurityAddon),
    paths(
        // Health checks
        crate::routes::checks::health_checker_handler,
        crate::routes::checks::database_health_checker,
        // Auth
        crate::routes::auth::authenticate,
        crate::routes::auth::register,
        crate::routes::auth::logout,
        crate::routes::auth::validate,
        crate::routes::auth::forgot_password,
        crate::routes::auth::change_email,
        crate::routes::auth::change_password,
        crate::routes::auth::change_pincode,
        crate::routes::auth::reset_pincode,
        crate::routes::auth::validate_username,
        crate::routes::auth::validate_email,
        // User
        crate::routes::user::profile,
        // News
        crate::routes::news::get_news,
        // Events (public)
        crate::routes::events::event_health,
        crate::routes::events::list_events,
        crate::routes::events::get_event_by_slug,
        crate::routes::events::registration::get_eligible_characters,
        crate::routes::events::registration::register_character,
        crate::routes::events::registration::register_guild,
        crate::routes::events::registration::get_registration_status,
        crate::routes::events::registration::withdraw_registration,
        crate::routes::events::registration::get_guild_members,
        crate::routes::events::match_status::get_match_status,
        crate::routes::events::match_status::get_matches,
        crate::routes::events::leaderboard::generic_leaderboard,
        crate::routes::events::leaderboard::guild_leaderboard,
        crate::routes::events::leaderboard::school_leaderboard,
        crate::routes::events::leaderboard::koth_leaderboard,
        crate::routes::events::participants::participant_characters,
        crate::routes::events::participants::participant_guilds,
        crate::routes::events::snapshots::list_snapshots,
        crate::routes::events::snapshots::snapshot_guild_scores,
        crate::routes::events::snapshots::snapshot_koth_scores,
        crate::routes::events::snapshots::snapshot_level_cap_race_scores,
        crate::routes::events::snapshots::snapshot_school_scores,
        // Admin
        crate::routes::admin::auth::authenticate,
        crate::routes::admin::profile::get_profile,
        // Admin - Players
        crate::routes::admin::players::get_players,
        crate::routes::admin::players::get_player_by_id,
        crate::routes::admin::players::ban_player_account,
        crate::routes::admin::players::ban_account_history,
        crate::routes::admin::players::account_transaction_history,
        crate::routes::admin::players::get_characters_by_account,
        crate::routes::admin::players::get_login_history,
        // Admin - Characters
        crate::routes::admin::characters::get_characters,
        crate::routes::admin::characters::get_character_by_id,
        // Admin - Item Shop
        crate::routes::admin::item_shop::get_shop_items,
        // Admin - Dashboard
        crate::routes::admin::dashboard::get_online_count,
        crate::routes::admin::dashboard::get_registered_count,
        crate::routes::admin::dashboard::get_transaction_count,
        crate::routes::admin::dashboard::get_recharge_total,
        // Admin - Events
        crate::routes::admin::events::crud::create_event,
        crate::routes::admin::events::crud::list_events,
        crate::routes::admin::events::crud::get_event_dashboard,
        crate::routes::admin::events::crud::activate_event,
        crate::routes::admin::events::crud::deactivate_event,
        crate::routes::admin::events::crud::get_event_detail,
        crate::routes::admin::events::crud::update_event,
        crate::routes::admin::events::match_management::create_matches,
        crate::routes::admin::events::match_management::generate_matches,
        crate::routes::admin::events::match_management::list_matches,
        crate::routes::admin::events::match_management::update_match,
        crate::routes::admin::events::match_management::delete_match,
        crate::routes::admin::events::gimmicks::create_gimmick,
        crate::routes::admin::events::gimmicks::list_gimmicks,
        crate::routes::admin::events::gimmicks::delete_gimmick,
        crate::routes::admin::events::participants::list_participants_by_character,
        crate::routes::admin::events::participants::list_participants_by_guild,
        crate::routes::admin::events::registration::admin_update_registration,
        crate::routes::admin::events::registration::admin_remove_participant,
        crate::routes::admin::events::export::export_characters_csv,
        crate::routes::admin::events::export::export_guilds_csv,
        crate::routes::admin::events::export::export_guild_match_results_csv,
        crate::routes::admin::events::export::export_koth_match_results_csv,
        crate::routes::admin::events::deductions::list_deductions,
        crate::routes::admin::events::deductions::apply_deduction,
        crate::routes::admin::events::deductions::revert_deduction,
        crate::routes::admin::events::tally::retally_match,
        crate::routes::admin::events::tally::set_tower_results,
        crate::routes::admin::events::tally::set_match_waiting,
        crate::routes::admin::events::tally::force_tally_level_cap_race,
        // Admin - Events - Leaderboards
        crate::routes::admin::events::leaderboard::admin_generic_leaderboard,
        crate::routes::admin::events::leaderboard::admin_guild_leaderboard,
        crate::routes::admin::events::leaderboard::admin_school_leaderboard,
        crate::routes::admin::events::leaderboard::admin_koth_leaderboard,
    ),
    components(
        schemas(
            // Auth requests
            AuthRequest,
            RegistrationRequest,
            ForgotPasswordRequest,
            ChangeEmailRequest,
            ChangePasswordRequest,
            ChangePincodeRequest,
            ValidateUsernameRequest,
            ValidateEmailRequest,
            // Admin auth
            AuthAdminRequest,
            AuthAdminProfileData,
            // Admin - Players
            BanAccountRequestPayload,
            UserAccountInfo,
            UserCharacterList,
            BanAccountHistory,
            UserLoginHistoryRecord,
            // Admin - Characters
            CharacterInfo,
            CharacterInfoList,
            // Admin - Item Shop
            GetItemShopData,
            ItemShopData,
            ItemShopBase,
            ItemGalleryImage,
            // News
            NewsItem,
            // Events - Registration
            RegisterRequest,
            GvgRegisterRequest,
            // Events - CRUD
            CreateEventRequest,
            UpdateEventRequest,
            // Events - Match Management
            CreateMatchRequest,
            CreateMatchesRequest,
            GenerateMatchesRequest,
            UpdateMatchRequest,
            // Events - Gimmicks
            CreateGimmickRequest,
            // Events - Deductions
            DeductionRequest,
            // Events - Tally
            SetTowerResultsRequest,
            // Pagination
            PaginationParams,
        )
    )
)]
pub struct ApiDoc;
