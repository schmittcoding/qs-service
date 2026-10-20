use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct AuthAdminRequest {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    pub username: String,
    #[validate(length(
        min = 3,
        max = 20,
        message = "Password must be between 3 and 20 characters"
    ))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, utoipa::ToSchema)]
pub struct AuthAdminProfileData {
    pub user_name: String,
    pub user_id: String,
    pub user_type: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // AuthAdminRequest Validation Tests
    // ========================================================================

    #[test]
    fn test_valid_admin_request() {
        let req = AuthAdminRequest {
            username: "admin123".to_string(),
            password: "password123".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_admin_request_username_too_short() {
        let req = AuthAdminRequest {
            username: "ab".to_string(), // 2 chars, min is 3
            password: "password123".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_admin_request_username_exactly_min_length() {
        let req = AuthAdminRequest {
            username: "abc".to_string(), // exactly 3 chars
            password: "password123".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_admin_request_username_exactly_max_length() {
        let req = AuthAdminRequest {
            username: "a".repeat(20), // exactly 20 chars
            password: "password123".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_admin_request_username_exceeds_max_length() {
        let req = AuthAdminRequest {
            username: "a".repeat(21), // 21 chars, max is 20
            password: "password123".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_admin_request_password_too_short() {
        let req = AuthAdminRequest {
            username: "admin123".to_string(),
            password: "ab".to_string(), // 2 chars, min is 3
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_admin_request_password_exactly_min_length() {
        let req = AuthAdminRequest {
            username: "admin123".to_string(),
            password: "abc".to_string(), // exactly 3 chars
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_admin_request_password_exactly_max_length() {
        let req = AuthAdminRequest {
            username: "admin123".to_string(),
            password: "p".repeat(20), // exactly 20 chars
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_admin_request_password_exceeds_max_length() {
        let req = AuthAdminRequest {
            username: "admin123".to_string(),
            password: "p".repeat(21), // 21 chars, max is 20
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_admin_request_empty_username() {
        let req = AuthAdminRequest {
            username: String::new(),
            password: "password123".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_admin_request_empty_password() {
        let req = AuthAdminRequest {
            username: "admin123".to_string(),
            password: String::new(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_admin_request_both_empty() {
        let req = AuthAdminRequest {
            username: String::new(),
            password: String::new(),
        };
        assert!(req.validate().is_err());
    }

    // ========================================================================
    // AuthAdminProfileData Tests
    // ========================================================================

    #[test]
    fn test_profile_data_creation() {
        let profile = AuthAdminProfileData {
            user_name: "admin1".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            user_type: 1,
        };
        assert_eq!(profile.user_name, "admin1");
        assert_eq!(profile.user_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(profile.user_type, 1);
    }

    #[test]
    fn test_profile_data_equality() {
        let profile1 = AuthAdminProfileData {
            user_name: "admin1".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            user_type: 1,
        };
        let profile2 = AuthAdminProfileData {
            user_name: "admin1".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            user_type: 1,
        };
        assert_eq!(profile1, profile2);
    }

    #[test]
    fn test_profile_data_inequality_different_user_type() {
        let profile1 = AuthAdminProfileData {
            user_name: "admin1".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            user_type: 1,
        };
        let profile2 = AuthAdminProfileData {
            user_name: "admin1".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            user_type: 2,
        };
        assert_ne!(profile1, profile2);
    }

    #[test]
    fn test_profile_data_serialization() {
        let profile = AuthAdminProfileData {
            user_name: "admin1".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            user_type: 1,
        };
        let json = serde_json::to_value(&profile).expect("should serialize");
        assert_eq!(json["user_name"], "admin1");
        assert_eq!(json["user_id"], "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(json["user_type"], 1);
    }

    #[test]
    fn test_profile_data_deserialization() {
        let json_str = r#"{
            "user_name": "admin1",
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "user_type": 2
        }"#;
        let profile: AuthAdminProfileData = serde_json::from_str(json_str).expect("should deserialize");
        assert_eq!(profile.user_name, "admin1");
        assert_eq!(profile.user_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(profile.user_type, 2);
    }

    #[test]
    fn test_profile_data_serde_roundtrip() {
        let original = AuthAdminProfileData {
            user_name: "superadmin".to_string(),
            user_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            user_type: 3,
        };
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: AuthAdminProfileData = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_profile_data_with_various_user_types() {
        let user_types = vec![0, 1, 2, 3, 255, -1];
        for user_type in user_types {
            let profile = AuthAdminProfileData {
                user_name: "admin".to_string(),
                user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                user_type,
            };
            assert_eq!(profile.user_type, user_type);
        }
    }

    #[test]
    fn test_profile_data_clone() {
        let original = AuthAdminProfileData {
            user_name: "admin1".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            user_type: 1,
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_profile_data_empty_strings() {
        let profile = AuthAdminProfileData {
            user_name: String::new(),
            user_id: String::new(),
            user_type: 0,
        };
        assert_eq!(profile.user_name, "");
        assert_eq!(profile.user_id, "");
    }
}