use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use super::plan_service::{OfferRepositoryForService, OfferData};

/// Offers service trait matching the original C# OffersService
#[async_trait]
pub trait OffersServiceTrait: Send + Sync {
    async fn get_offers(&self) -> Result<Vec<OffersModel>, String>;
    async fn get_offer_by_id(&self, offer_guid: Uuid) -> Result<OfferModel, String>;
}

/// Offers Model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OffersModel {
    pub id: i32,
    pub offer_id: String,
    pub offer_name: Option<String>,
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    pub user_id: Option<i32>,
    pub offer_guid: Uuid,
}

/// Offer Model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OfferModel {
    pub id: i32,
    pub offer_id: String,
    pub offer_name: Option<String>,
    pub offer_guid: Uuid,
}

/// Concrete implementation of OffersService
pub struct OffersServiceImpl {
    offer_repo: Arc<dyn OfferRepositoryForService>,
}

impl OffersServiceImpl {
    pub fn new(offer_repo: Arc<dyn OfferRepositoryForService>) -> Self {
        Self { offer_repo }
    }
}

#[async_trait]
impl OffersServiceTrait for OffersServiceImpl {
    async fn get_offers(&self) -> Result<Vec<OffersModel>, String> {
        let offers = self.offer_repo.get_all().await?;
        Ok(offers
            .into_iter()
            .map(|offer| OffersModel {
                id: offer.id,
                offer_id: offer.offer_id,
                offer_name: offer.offer_name.clone(),
                create_date: offer.create_date,
                user_id: offer.user_id,
                offer_guid: offer.offer_guid,
            })
            .collect())
    }

    async fn get_offer_by_id(&self, offer_guid: Uuid) -> Result<OfferModel, String> {
        let offer = self
            .offer_repo
            .get_offer_by_guid(offer_guid)
            .await?
            .ok_or_else(|| "Offer not found".to_string())?;

        Ok(OfferModel {
            id: offer.id,
            offer_id: offer.offer_id,
            offer_name: offer.offer_name.clone(),
            offer_guid: offer.offer_guid,
        })
    }
}

