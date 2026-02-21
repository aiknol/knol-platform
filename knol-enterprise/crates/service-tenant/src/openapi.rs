//! OpenAPI specification and Swagger UI for the tenant service.

use utoipa::OpenApi;

use crate::auth::AppClaims;
use crate::routes::{app, billing, invites, settings};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Knol Tenant API",
        version = "1.0.0",
        description = "Self-service API for tenant workspaces: authentication, billing, team management, API keys, and settings.",
        contact(name = "Knol", url = "https://aiknol.com")
    ),
    paths(
        // Auth
        app::signup,
        app::login,
        app::logout,
        app::me,
        // Tenant / Users
        app::tenant,
        app::list_users,
        app::create_user,
        app::update_user,
        // API Keys
        app::list_api_keys,
        app::create_api_key,
        app::revoke_api_key,
        // Audit
        app::list_audit_logs,
        // Billing
        billing::create_checkout,
        billing::create_portal,
        billing::get_subscription,
        billing::cancel_subscription,
        billing::reactivate_subscription,
        billing::list_invoices,
        billing::upcoming_invoice,
        billing::get_usage,
        billing::get_usage_history,
        billing::stripe_webhook,
        // Invites
        invites::create_invite,
        invites::list_invites,
        invites::revoke_invite,
        invites::accept_invite,
        // Settings
        settings::update_tenant_settings,
        settings::update_profile,
        settings::change_password,
    ),
    components(
        schemas(
            AppClaims,
            // App
            app::SignupRequest,
            app::LoginRequest,
            app::CreateApiKeyRequest,
            app::CreateAppUserRequest,
            app::UpdateAppUserRequest,
            // Billing
            billing::CheckoutRequest,
            // Invites
            invites::CreateInviteRequest,
            invites::AcceptInviteRequest,
            // Settings
            settings::UpdateTenantRequest,
            settings::UpdateProfileRequest,
            settings::ChangePasswordRequest,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Authentication: signup, login, logout, profile"),
        (name = "Users", description = "Tenant workspace and user management"),
        (name = "API Keys", description = "API key lifecycle"),
        (name = "Billing", description = "Stripe subscriptions, invoices, checkout"),
        (name = "Usage", description = "Usage tracking and alerts"),
        (name = "Invites", description = "Team invitations"),
        (name = "Settings", description = "Tenant and profile settings"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}
