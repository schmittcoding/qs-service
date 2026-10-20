pub mod http_messages {
    pub mod success {
        pub const DEFAULT: &str = "Request completed successfully.";
        pub const ADD: &str = "Successfully added.";
        pub const EDIT: &str = "Successfully updated.";
        pub const DELETE: &str = "Successfully deleted.";
    }

    pub mod error {
        pub const NOT_FOUND: &str = "The requested resource was not found.";
        pub const UNAUTHORIZED: &str = "Unauthorized access. You do not have permission to perform this action.";
        pub const FORBIDDEN: &str = "Access denied. You are not allowed to perform this operation.";
        pub const BAD_REQUEST: &str = "The request was invalid or cannot be processed.";
        pub const INTERNAL_SERVER_ERROR: &str = "An unexpected error occurred. Please try again later.";
        pub const VALIDATION_FAILED: &str = "One or more fields have invalid values. Please check your input.";
        pub const CONFLICT: &str = "The request could not be completed due to a conflict.";
        pub const TIMEOUT: &str = "The request timed out. Please try again.";
        pub const TOO_MANY_REQUESTS: &str = "Too many requests. Please try again later.";
        pub const DUPLICATE_DATA: &str = "Data already exists. Duplicate entry is not allowed.";
    }
}
