use async_trait::async_trait;
use crate::models::Offer;
use crate::pool::DbPool;
use uuid::Uuid;

#[async_trait]
pub trait OfferRepository: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<Option<Offer>, sqlx::Error>;
    async fn get_by_offer_guid(&self, offer_guid: Uuid) -> Result<Option<Offer>, sqlx::Error>;
    async fn get_by_offer_id(&self, offer_id: &str) -> Result<Option<Offer>, sqlx::Error>;
    async fn get_all(&self) -> Result<Vec<Offer>, sqlx::Error>;
}

pub struct PostgresOfferRepository {
    pool: DbPool,
}

impl PostgresOfferRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OfferRepository for PostgresOfferRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<Offer>, sqlx::Error> {
        sqlx::query_as::<_, Offer>(
            "SELECT id, offer_id, offer_name, offer_guid, create_date, user_id FROM offers WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&{self.pool.get()})
        .await
    }

    async fn get_by_offer_guid(&self, offer_guid: Uuid) -> Result<Option<Offer>, sqlx::Error> {
        sqlx::query_as::<_, Offer>(
            "SELECT id, offer_id, offer_name, offer_guid, create_date, user_id FROM offers WHERE offer_guid = $1",
        )
        .bind(offer_guid)
        .fetch_optional(&{self.pool.get()})
        .await
    }

    async fn get_by_offer_id(&self, offer_id: &str) -> Result<Option<Offer>, sqlx::Error> {
        sqlx::query_as::<_, Offer>(
            "SELECT id, offer_id, offer_name, offer_guid, create_date, user_id FROM offers WHERE offer_id = $1",
        )
        .bind(offer_id)
        .fetch_optional(&{self.pool.get()})
        .await
    }

    async fn get_all(&self) -> Result<Vec<Offer>, sqlx::Error> {
        sqlx::query_as::<_, Offer>(
            "SELECT id, offer_id, offer_name, offer_guid, create_date, user_id FROM offers",
        )
        .fetch_all(&{self.pool.get()})
        .await
    }
}

