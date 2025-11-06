use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;

/// Plan service trait matching the original C# PlanService
#[async_trait]
pub trait PlanServiceTrait: Send + Sync {
    async fn get_plans(&self) -> Result<Vec<PlansModel>, String>;
    async fn get_plan_detail_by_plan_guid(&self, plan_guid: Uuid) -> Result<PlansModel, String>;
    async fn get_metered_plans(&self) -> Result<Vec<PlansModel>, String>;
    async fn get_plans_model_by_amp_plan_id_offer_id(
        &self,
        amp_plan_id: &str,
        amp_offer_id: &str,
    ) -> Result<PlansModel, String>;
}

/// Plans Model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlansModel {
    pub id: i32,
    pub plan_id: String,
    pub display_name: String,
    pub description: String,
    pub is_metering_supported: Option<bool>,
    pub offer_id: Option<Uuid>,
    pub offer_name: Option<String>,
    pub plan_guid: Uuid,
}

/// Plan data for internal use
#[derive(Debug, Clone)]
pub struct PlanData {
    pub id: i32,
    pub plan_id: String,
    pub description: Option<String>,
    pub display_name: Option<String>,
    pub is_metering_supported: Option<bool>,
    pub is_per_user: Option<bool>,
    pub plan_guid: Uuid,
    pub offer_id: Uuid,
}

/// Plan repository trait for dependency injection
#[async_trait]
pub trait PlanRepositoryForService: Send + Sync {
    async fn get_plans_by_user(&self) -> Result<Vec<PlanData>, String>;
    async fn get_by_internal_reference(&self, plan_guid: Uuid) -> Result<Option<PlanData>, String>;
    async fn get_all(&self) -> Result<Vec<PlanData>, String>;
}

/// Offer repository trait for dependency injection  
#[async_trait]
pub trait OfferRepositoryForService: Send + Sync {
    async fn get_all(&self) -> Result<Vec<OfferData>, String>;
    async fn get_offer_by_guid(&self, offer_guid: Uuid) -> Result<Option<OfferData>, String>;
    async fn get_offer_by_offer_id(&self, offer_id: &str) -> Result<Option<OfferData>, String>;
}

/// Offer data for internal use
#[derive(Debug, Clone)]
pub struct OfferData {
    pub id: i32,
    pub offer_id: String,
    pub offer_name: Option<String>,
    pub offer_guid: Uuid,
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    pub user_id: Option<i32>,
}

/// Concrete implementation of PlanService
pub struct PlanServiceImpl {
    plan_repo: Arc<dyn PlanRepositoryForService>,
    offer_repo: Arc<dyn OfferRepositoryForService>,
}

impl PlanServiceImpl {
    pub fn new(
        plan_repo: Arc<dyn PlanRepositoryForService>,
        offer_repo: Arc<dyn OfferRepositoryForService>,
    ) -> Self {
        Self {
            plan_repo,
            offer_repo,
        }
    }
}

#[async_trait]
impl PlanServiceTrait for PlanServiceImpl {
    async fn get_plans(&self) -> Result<Vec<PlansModel>, String> {
        let all_plans = self.plan_repo.get_plans_by_user().await?;
        let offer_details = self.offer_repo.get_all().await?;

        let mut plans_list: Vec<PlansModel> = all_plans
            .into_iter()
            .map(|plan| PlansModel {
                id: plan.id,
                plan_id: plan.plan_id,
                display_name: plan.display_name.unwrap_or_default(),
                description: plan.description.unwrap_or_default(),
                is_metering_supported: plan.is_metering_supported,
                offer_id: Some(plan.offer_id),
                offer_name: None,
                plan_guid: plan.plan_guid,
            })
            .collect();

        for plan in &mut plans_list {
            if let Some(offer_id) = plan.offer_id {
                if let Some(offer) = offer_details.iter().find(|o| o.offer_guid == offer_id) {
                    plan.offer_name = offer.offer_name.clone();
                }
            }
        }

        Ok(plans_list)
    }

    async fn get_plan_detail_by_plan_guid(&self, plan_guid: Uuid) -> Result<PlansModel, String> {
        let existing_plan = self
            .plan_repo
            .get_by_internal_reference(plan_guid)
            .await?
            .ok_or_else(|| "Plan not found".to_string())?;

        let offer_details = self
            .offer_repo
            .get_offer_by_guid(existing_plan.offer_id)
            .await?;

        Ok(PlansModel {
            id: existing_plan.id,
            plan_id: existing_plan.plan_id,
            display_name: existing_plan.display_name.unwrap_or_default(),
            description: existing_plan.description.unwrap_or_default(),
            is_metering_supported: existing_plan.is_metering_supported,
            offer_id: Some(existing_plan.offer_id),
            offer_name: offer_details.and_then(|o| o.offer_name),
            plan_guid: existing_plan.plan_guid,
        })
    }

    async fn get_metered_plans(&self) -> Result<Vec<PlansModel>, String> {
        let all_plans = self.get_plans().await?;
        Ok(all_plans
            .into_iter()
            .filter(|p| p.is_metering_supported == Some(true))
            .collect())
    }

    async fn get_plans_model_by_amp_plan_id_offer_id(
        &self,
        amp_plan_id: &str,
        amp_offer_id: &str,
    ) -> Result<PlansModel, String> {
        let offer = self
            .offer_repo
            .get_offer_by_offer_id(amp_offer_id)
            .await?
            .ok_or_else(|| "Offer not found".to_string())?;

        let all_plans = self.plan_repo.get_all().await?;
        let plan = all_plans
            .into_iter()
            .find(|p| p.plan_id == amp_plan_id && p.offer_id == offer.offer_guid)
            .ok_or_else(|| "Plan not found".to_string())?;

        Ok(PlansModel {
            id: plan.id,
            plan_id: plan.plan_id,
            display_name: plan.display_name.unwrap_or_default(),
            description: plan.description.unwrap_or_default(),
            is_metering_supported: plan.is_metering_supported,
            offer_id: Some(plan.offer_id),
            offer_name: offer.offer_name.clone(),
            plan_guid: plan.plan_guid,
        })
    }
}

