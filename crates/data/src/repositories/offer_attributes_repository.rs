use async_trait::async_trait;
use crate::models::OfferAttributes;
use crate::pool::DbPool;
use uuid::Uuid;

#[async_trait]
pub trait OfferAttributesRepository: Send + Sync {
    async fn add(&self, offer_attributes: &OfferAttributes) -> Result<Option<i32>, sqlx::Error>;
    async fn get_all(&self) -> Result<Vec<OfferAttributes>, sqlx::Error>;
    async fn get_input_attributes_by_offer_id(
        &self,
        offer_id: Uuid,
    ) -> Result<Vec<OfferAttributes>, sqlx::Error>;
    async fn get_by_id(&self, offer_attribute_id: i32) -> Result<Option<OfferAttributes>, sqlx::Error>;
    async fn get_all_offer_attributes_by_offer_id(
        &self,
        offer_guid: Uuid,
    ) -> Result<Vec<OfferAttributes>, sqlx::Error>;
    async fn delete(&self, offer_attribute_id: i32) -> Result<(), sqlx::Error>;
}

pub struct PostgresOfferAttributesRepository {
    pool: DbPool,
}

impl PostgresOfferAttributesRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OfferAttributesRepository for PostgresOfferAttributesRepository {
    async fn add(&self, offer_attributes: &OfferAttributes) -> Result<Option<i32>, sqlx::Error> {
        let existing = sqlx::query_as::<_, OfferAttributes>(
            "SELECT id, offer_id, parameter_id, display_name, description, type, values_list, create_date
             FROM offer_attributes WHERE id = $1",
        )
        .bind(offer_attributes.id)
        .fetch_optional(&{self.pool.get()})
        .await?;

        if let Some(existing_attr) = existing {
            sqlx::query(
                "UPDATE offer_attributes SET parameter_id = $2, display_name = $3, description = $4,
                 type = $5, values_list = $6 WHERE id = $1",
            )
            .bind(existing_attr.id)
            .bind(&offer_attributes.parameter_id)
            .bind(&offer_attributes.display_name)
            .bind(&offer_attributes.description)
            .bind(&offer_attributes.type_)
            .bind(&offer_attributes.values_list)
            .execute(&{self.pool.get()})
            .await?;
            Ok(Some(existing_attr.id))
        } else {
            let result = sqlx::query_as::<_, OfferAttributes>(
                "INSERT INTO offer_attributes (offer_id, parameter_id, display_name, description, type, values_list, create_date)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 RETURNING id, offer_id, parameter_id, display_name, description, type, values_list, create_date",
            )
            .bind(offer_attributes.offer_id)
            .bind(&offer_attributes.parameter_id)
            .bind(&offer_attributes.display_name)
            .bind(&offer_attributes.description)
            .bind(&offer_attributes.type_)
            .bind(&offer_attributes.values_list)
            .bind(chrono::Utc::now())
            .fetch_one(&{self.pool.get()})
            .await?;
            Ok(Some(result.id))
        }
    }

    async fn get_all(&self) -> Result<Vec<OfferAttributes>, sqlx::Error> {
        sqlx::query_as::<_, OfferAttributes>(
            "SELECT id, offer_id, parameter_id, display_name, description, type, values_list, create_date
             FROM offer_attributes",
        )
        .fetch_all(&{self.pool.get()})
        .await
    }

    async fn get_input_attributes_by_offer_id(
        &self,
        offer_id: Uuid,
    ) -> Result<Vec<OfferAttributes>, sqlx::Error> {
        sqlx::query_as::<_, OfferAttributes>(
            "SELECT oa.id, oa.offer_id, oa.parameter_id, oa.display_name, oa.description, oa.type, oa.values_list, oa.create_date
             FROM offer_attributes oa
             INNER JOIN offers o ON oa.offer_id = o.id
             WHERE o.offer_guid = $1 AND oa.type = 'input'",
        )
        .bind(offer_id)
        .fetch_all(&{self.pool.get()})
        .await
    }

    async fn get_by_id(&self, offer_attribute_id: i32) -> Result<Option<OfferAttributes>, sqlx::Error> {
        sqlx::query_as::<_, OfferAttributes>(
            "SELECT id, offer_id, parameter_id, display_name, description, type, values_list, create_date
             FROM offer_attributes WHERE id = $1",
        )
        .bind(offer_attribute_id)
        .fetch_optional(&{self.pool.get()})
        .await
    }

    async fn get_all_offer_attributes_by_offer_id(
        &self,
        offer_guid: Uuid,
    ) -> Result<Vec<OfferAttributes>, sqlx::Error> {
        sqlx::query_as::<_, OfferAttributes>(
            "SELECT oa.id, oa.offer_id, oa.parameter_id, oa.display_name, oa.description, oa.type, oa.values_list, oa.create_date
             FROM offer_attributes oa
             INNER JOIN offers o ON oa.offer_id = o.id
             WHERE o.offer_guid = $1",
        )
        .bind(offer_guid)
        .fetch_all(&{self.pool.get()})
        .await
    }

    async fn delete(&self, offer_attribute_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM offer_attributes WHERE id = $1")
            .bind(offer_attribute_id)
            .execute(&{self.pool.get()})
            .await?;
        Ok(())
    }
}

