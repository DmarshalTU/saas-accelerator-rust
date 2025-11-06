use async_trait::async_trait;
use crate::models::*;
use crate::pool::DbPool;
use uuid::Uuid;

pub mod subscription_repository;
pub mod user_repository;
pub mod plan_repository;
pub mod offer_repository;
pub mod application_config_repository;
pub mod subscription_audit_log_repository;
pub mod metered_plan_scheduler_repository;
pub mod application_log_repository;
pub mod email_template_repository;
pub mod events_repository;
pub mod plan_events_mapping_repository;
pub mod offer_attributes_repository;

pub use subscription_repository::{SubscriptionRepository, PostgresSubscriptionRepository};
pub use user_repository::{UserRepository, PostgresUserRepository};
pub use plan_repository::{PlanRepository, PostgresPlanRepository};
pub use offer_repository::{OfferRepository, PostgresOfferRepository};
pub use application_config_repository::{ApplicationConfigRepository, PostgresApplicationConfigRepository};
pub use subscription_audit_log_repository::{SubscriptionAuditLogRepository, PostgresSubscriptionAuditLogRepository};
pub use metered_plan_scheduler_repository::{MeteredPlanSchedulerRepository, PostgresMeteredPlanSchedulerRepository};
pub use application_log_repository::{ApplicationLogRepository, PostgresApplicationLogRepository};
pub use email_template_repository::{EmailTemplateRepository, PostgresEmailTemplateRepository};
pub use events_repository::{EventsRepository, PostgresEventsRepository};
pub use plan_events_mapping_repository::{PlanEventsMappingRepository, PostgresPlanEventsMappingRepository};
pub use offer_attributes_repository::{OfferAttributesRepository, PostgresOfferAttributesRepository};

