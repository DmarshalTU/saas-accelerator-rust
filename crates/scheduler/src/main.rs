use chrono::{DateTime, Utc, Duration, Timelike};
use data::{
    pool::create_pool,
    repositories::{
        ApplicationConfigRepository, MeteredPlanSchedulerRepository, PlanRepository,
        PostgresApplicationConfigRepository, PostgresMeteredPlanSchedulerRepository,
        PostgresPlanRepository, PostgresSubscriptionRepository, SubscriptionRepository,
    },
};
use marketplace::{client::MarketplaceClient, metering::MeteringApiClient};
use shared::models::MeteringUsageRequest;
use std::env;
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenv::dotenv().ok();

    info!("Starting Metered Billing Scheduler");

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).await?;

    let config_repo: Arc<dyn ApplicationConfigRepository> =
        Arc::new(PostgresApplicationConfigRepository::new(pool.clone()));
    let scheduler_repo: Arc<dyn MeteredPlanSchedulerRepository> =
        Arc::new(PostgresMeteredPlanSchedulerRepository::new(pool.clone()));
    let subscription_repo: Arc<dyn SubscriptionRepository> =
        Arc::new(PostgresSubscriptionRepository::new(pool.clone()));
    let plan_repo: Arc<dyn PlanRepository> =
        Arc::new(PostgresPlanRepository::new(pool.clone()));

    let marketplace_base_url = env::var("MARKETPLACE_API_BASE_URL")
        .unwrap_or_else(|_| "https://marketplaceapi.microsoft.com/api".to_string());
    let api_version = env::var("MARKETPLACE_API_VERSION")
        .unwrap_or_else(|_| "2018-08-31".to_string());

    let marketplace_client = MarketplaceClient::builder(marketplace_base_url)
        .with_client_secret(
            &env::var("SaaS_API_TENANT_ID").unwrap_or_default(),
            &env::var("SaaS_API_CLIENT_ID").unwrap_or_default(),
            &env::var("SaaS_API_CLIENT_SECRET").unwrap_or_default(),
        )
        .build();

    let metering_client = MeteringApiClient::new(marketplace_client, api_version);

    let is_metered_billing_enabled = config_repo.get_by_name("IsMeteredBillingEnabled")
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| "false".to_string());
    
    if is_metered_billing_enabled != "true" {
        info!("Metered billing is disabled. Exiting.");
        return Ok(());
    }

    info!("Executing scheduler...");

    let current_time = Utc::now();
    let current_hour = current_time.date_naive().and_hms_opt(
        current_time.hour(),
        0,
        0,
    ).unwrap();
    let current_utc_time = DateTime::from_naive_utc_and_offset(current_hour, Utc);

    let frequencies = vec![("Hourly", 1), ("Daily", 2), ("Weekly", 3), ("Monthly", 4), ("Yearly", 5), ("OneTime", 6)];

    for (frequency_name, frequency_id) in frequencies {
        let enable_key = format!("Enable{frequency_name}MeterSchedules");
        let is_enabled = config_repo.get_by_name(&enable_key)
            .await
            .unwrap_or(None)
            .unwrap_or_else(|| "false".to_string());

        if is_enabled != "true" {
            info!("{} scheduled items are disabled", frequency_name);
            continue;
        }

        info!("Checking all {} scheduled items at {} UTC", frequency_name, current_utc_time);

        let scheduled_items = get_scheduled_items(
            &scheduler_repo,
            &subscription_repo,
            &plan_repo,
            frequency_id,
            frequency_name,
        ).await?;

        for item in scheduled_items {
            process_scheduled_item(
                &pool,
                &metering_client,
                &scheduler_repo,
                &item,
                current_utc_time,
            ).await?;
        }
    }

    info!("Scheduler execution completed");
    Ok(())
}


#[derive(Debug)]
struct ScheduledItem {
    id: i32,
    subscription_id: uuid::Uuid,
    plan_id: String,
    dimension: String,
    quantity: f64,
    start_date: DateTime<Utc>,
    next_run_time: Option<DateTime<Utc>>,
    frequency: String,
}

async fn get_scheduled_items(
    scheduler_repo: &Arc<dyn MeteredPlanSchedulerRepository>,
    subscription_repo: &Arc<dyn SubscriptionRepository>,
    plan_repo: &Arc<dyn PlanRepository>,
    frequency_id: i32,
    frequency_name: &str,
) -> Result<Vec<ScheduledItem>, sqlx::Error> {
    let schedulers = scheduler_repo.get_all().await?;

    let mut items = Vec::new();
    for scheduler in schedulers {
        if scheduler.frequency_id == frequency_id {
            let subscription = subscription_repo.get_by_id(scheduler.subscription_id).await?;
            let plan = plan_repo.get_by_id(scheduler.plan_id).await?;

            if let (Some(sub), Some(pl)) = (subscription, plan) {
                items.push(ScheduledItem {
                    id: scheduler.id,
                    subscription_id: sub.amp_subscription_id,
                    plan_id: pl.plan_id,
                    dimension: String::new(),
                    quantity: scheduler.quantity,
                    start_date: scheduler.start_date,
                    next_run_time: scheduler.next_run_time,
                    frequency: frequency_name.to_string(),
                });
            }
        }
    }

    Ok(items)
}

async fn process_scheduled_item(
    pool: &data::pool::DbPool,
    metering_client: &MeteringApiClient,
    scheduler_repo: &Arc<dyn MeteredPlanSchedulerRepository>,
    item: &ScheduledItem,
    current_time: DateTime<Utc>,
) -> Result<(), Box<dyn std::error::Error>> {
    let next_run_time = item.next_run_time.unwrap_or(item.start_date);
    let time_diff_hours = (current_time - next_run_time).num_hours();

    info!(
        "Scheduled Item Id: {} - NextRun: {} - TimeDiff: {} hours",
        item.id, next_run_time, time_diff_hours
    );

    if time_diff_hours > 0 {
        warn!(
            "Scheduled Item Id: {} will not run as {} has passed",
            item.id, next_run_time
        );
        return Ok(());
    }

    if time_diff_hours < 0 {
        info!(
            "Scheduled Item Id: {} future run will be at {} UTC",
            item.id, next_run_time
        );
        return Ok(());
    }

    if time_diff_hours == 0 {
        info!("Triggering scheduled item {}", item.id);
        trigger_scheduled_item(pool, metering_client, scheduler_repo, item).await?;
    }

    Ok(())
}

async fn trigger_scheduled_item(
    pool: &data::pool::DbPool,
    metering_client: &MeteringApiClient,
    scheduler_repo: &Arc<dyn MeteredPlanSchedulerRepository>,
    item: &ScheduledItem,
) -> Result<(), Box<dyn std::error::Error>> {
    let usage_request = MeteringUsageRequest {
        resource_id: item.subscription_id,
        plan_id: item.plan_id.clone(),
        dimension: item.dimension.clone(),
        quantity: item.quantity,
        effective_start_time: Utc::now(),
    };

    let request_json = serde_json::to_string(&usage_request)?;
    info!("Scheduled Item {} - Request: {}", item.id, request_json);

    match metering_client.emit_usage_event(&usage_request).await {
        Ok(result) => {
            let response_json = serde_json::to_string(&result)?;
            info!("Scheduled Item {} - Response: {}", item.id, response_json);

            // Save audit log
            save_audit_log(
                pool,
                item.id,
                &request_json,
                &response_json,
                &result.status,
                item.subscription_id,
            ).await?;

            if result.status == "Accepted" {
                let last_run = item.next_run_time.unwrap_or(item.start_date);
                if let Some(next_run) = calculate_next_run_time(last_run, &item.frequency) {
                    scheduler_repo.update_next_run_time(item.id, next_run).await?;
                }
            }
        }
        Err(e) => {
            error!("Scheduled Item {} - Error: {}", item.id, e);
            let error_json = format!("{{\"error\": \"{e}\"}}");
            save_audit_log(
                pool,
                item.id,
                &request_json,
                &error_json,
                "Error",
                item.subscription_id,
            ).await?;
        }
    }

    Ok(())
}

async fn save_audit_log(
    pool: &data::pool::DbPool,
    scheduler_id: i32,
    request_json: &str,
    response_json: &str,
    status: &str,
    subscription_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    // Get subscription database ID
    let pg = pool.get();
    let sub_db_id: Option<i32> = sqlx::query_scalar(
        "SELECT id FROM subscriptions WHERE amp_subscription_id = $1"
    )
    .bind(subscription_id)
    .fetch_optional(&pg)
    .await?;

    if let Some(sub_id) = sub_db_id {
        sqlx::query(
            r"
            INSERT INTO metered_audit_logs 
            (subscription_id, request_json, response_json, status_code, created_date, subscription_usage_date, run_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "
        )
        .bind(sub_id)
        .bind(request_json)
        .bind(response_json)
        .bind(status)
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(format!("Scheduler-{scheduler_id}"))
        .execute(&pg)
        .await?;
    }

    Ok(())
}

pub(crate) fn calculate_next_run_time(start_date: DateTime<Utc>, frequency: &str) -> Option<DateTime<Utc>> {
    if frequency == "OneTime" {
        return None;
    }

    Some(match frequency {
        "Hourly" => start_date + Duration::hours(1),
        "Daily" => start_date + Duration::days(1),
        "Weekly" => start_date + Duration::days(7),
        "Monthly" => start_date + Duration::days(30),
        "Yearly" => start_date + Duration::days(365),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_next_run_time_one_time_returns_none() {
        let t = Utc::now();
        assert!(calculate_next_run_time(t, "OneTime").is_none());
    }

    #[test]
    fn calculate_next_run_time_hourly_adds_one_hour() {
        let t = DateTime::parse_from_rfc3339("2024-01-15T12:00:00Z").unwrap().with_timezone(&Utc);
        let next = calculate_next_run_time(t, "Hourly").unwrap();
        let expected = DateTime::parse_from_rfc3339("2024-01-15T13:00:00Z").unwrap().with_timezone(&Utc);
        assert_eq!(next, expected);
    }

    #[test]
    fn calculate_next_run_time_daily_adds_one_day() {
        let t = DateTime::parse_from_rfc3339("2024-01-15T12:00:00Z").unwrap().with_timezone(&Utc);
        let next = calculate_next_run_time(t, "Daily").unwrap();
        let expected = DateTime::parse_from_rfc3339("2024-01-16T12:00:00Z").unwrap().with_timezone(&Utc);
        assert_eq!(next, expected);
    }

    #[test]
    fn calculate_next_run_time_weekly_adds_seven_days() {
        let t = DateTime::parse_from_rfc3339("2024-01-15T12:00:00Z").unwrap().with_timezone(&Utc);
        let next = calculate_next_run_time(t, "Weekly").unwrap();
        let expected = DateTime::parse_from_rfc3339("2024-01-22T12:00:00Z").unwrap().with_timezone(&Utc);
        assert_eq!(next, expected);
    }

    #[test]
    fn calculate_next_run_time_unknown_returns_none() {
        let t = Utc::now();
        assert!(calculate_next_run_time(t, "Unknown").is_none());
        assert!(calculate_next_run_time(t, "").is_none());
    }
}

