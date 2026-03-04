-- Initial schema migration for SaaS Accelerator
-- Converted from Entity Framework Core migrations

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE IF NOT EXISTS users (
    user_id SERIAL PRIMARY KEY,
    email_address VARCHAR(100),
    created_date TIMESTAMP WITH TIME ZONE,
    full_name VARCHAR(200)
);

-- Offers table
CREATE TABLE IF NOT EXISTS offers (
    id SERIAL PRIMARY KEY,
    offer_id VARCHAR(225) NOT NULL UNIQUE,
    offer_name VARCHAR(225),
    offer_guid UUID NOT NULL UNIQUE DEFAULT uuid_generate_v4(),
    create_date TIMESTAMP WITH TIME ZONE,
    user_id INTEGER REFERENCES users(user_id)
);

-- Plans table
CREATE TABLE IF NOT EXISTS plans (
    id SERIAL PRIMARY KEY,
    plan_id VARCHAR(100) NOT NULL UNIQUE,
    description VARCHAR(500),
    display_name VARCHAR(100),
    is_metering_supported BOOLEAN,
    is_per_user BOOLEAN,
    plan_guid UUID NOT NULL DEFAULT uuid_generate_v4(),
    offer_id UUID NOT NULL REFERENCES offers(offer_guid)
);

-- Subscriptions table
CREATE TABLE IF NOT EXISTS subscriptions (
    id SERIAL PRIMARY KEY,
    amp_subscription_id UUID NOT NULL UNIQUE DEFAULT uuid_generate_v4(),
    subscription_status VARCHAR(50),
    amp_plan_id VARCHAR(100),
    amp_offer_id VARCHAR(225),
    is_active BOOLEAN,
    create_by INTEGER,
    create_date TIMESTAMP WITH TIME ZONE,
    modify_date TIMESTAMP WITH TIME ZONE,
    user_id INTEGER REFERENCES users(user_id),
    name VARCHAR(100),
    amp_quantity INTEGER NOT NULL DEFAULT 0,
    purchaser_email VARCHAR(225),
    purchaser_tenant_id UUID,
    term VARCHAR(50),
    start_date TIMESTAMP WITH TIME ZONE,
    end_date TIMESTAMP WITH TIME ZONE
);

-- Metered Dimensions table
CREATE TABLE IF NOT EXISTS metered_dimensions (
    id SERIAL PRIMARY KEY,
    plan_id INTEGER NOT NULL REFERENCES plans(id),
    dimension VARCHAR(150) NOT NULL,
    description VARCHAR(250),
    created_date TIMESTAMP WITH TIME ZONE
);

-- Metered Audit Logs table
CREATE TABLE IF NOT EXISTS metered_audit_logs (
    id SERIAL PRIMARY KEY,
    subscription_id INTEGER NOT NULL REFERENCES subscriptions(id),
    request_json VARCHAR(500),
    response_json VARCHAR(500),
    status_code VARCHAR(100),
    created_date TIMESTAMP WITH TIME ZONE,
    subscription_usage_date TIMESTAMP WITH TIME ZONE,
    run_by VARCHAR(255)
);

-- Subscription Audit Logs table
CREATE TABLE IF NOT EXISTS subscription_audit_logs (
    id SERIAL PRIMARY KEY,
    subscription_id INTEGER NOT NULL REFERENCES subscriptions(id),
    attribute VARCHAR(20),
    old_value VARCHAR(50),
    new_value TEXT,
    create_date TIMESTAMP WITH TIME ZONE,
    create_by INTEGER REFERENCES users(user_id)
);

-- Web Job Subscription Status table
CREATE TABLE IF NOT EXISTS web_job_subscription_status (
    id SERIAL PRIMARY KEY,
    subscription_id UUID REFERENCES subscriptions(amp_subscription_id),
    subscription_status VARCHAR(50),
    description TEXT,
    insert_date TIMESTAMP WITH TIME ZONE
);

-- Application Configuration table
CREATE TABLE IF NOT EXISTS application_configuration (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    value TEXT,
    description VARCHAR(255)
);

-- Known Users table
CREATE TABLE IF NOT EXISTS known_users (
    id SERIAL PRIMARY KEY,
    user_email VARCHAR(50) NOT NULL UNIQUE,
    role_id INTEGER NOT NULL
);

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE
);

-- Scheduler Frequency table
CREATE TABLE IF NOT EXISTS scheduler_frequency (
    id SERIAL PRIMARY KEY,
    frequency VARCHAR(50) NOT NULL UNIQUE
);

-- Metered Plan Scheduler Management table
CREATE TABLE IF NOT EXISTS metered_plan_scheduler_management (
    id SERIAL PRIMARY KEY,
    scheduler_name VARCHAR(50) NOT NULL,
    subscription_id INTEGER NOT NULL REFERENCES subscriptions(id),
    plan_id INTEGER NOT NULL REFERENCES plans(id),
    dimension_id INTEGER NOT NULL REFERENCES metered_dimensions(id),
    frequency_id INTEGER NOT NULL REFERENCES scheduler_frequency(id),
    quantity DOUBLE PRECISION NOT NULL,
    start_date TIMESTAMP WITH TIME ZONE NOT NULL,
    next_run_time TIMESTAMP WITH TIME ZONE
);

-- Application Log table
CREATE TABLE IF NOT EXISTS application_log (
    id SERIAL PRIMARY KEY,
    action_time TIMESTAMP WITH TIME ZONE,
    log_detail TEXT
);

-- Email Template table
CREATE TABLE IF NOT EXISTS email_template (
    id SERIAL PRIMARY KEY,
    status VARCHAR(1000),
    description VARCHAR(1000),
    insert_date TIMESTAMP WITH TIME ZONE,
    template_body TEXT,
    subject VARCHAR(1000),
    to_recipients VARCHAR(1000),
    cc VARCHAR(1000),
    bcc VARCHAR(1000),
    is_active BOOLEAN DEFAULT false
);

-- Events table
CREATE TABLE IF NOT EXISTS events (
    id SERIAL PRIMARY KEY,
    events_name VARCHAR(225),
    is_active BOOLEAN DEFAULT true,
    create_date TIMESTAMP WITH TIME ZONE
);

-- Plan Events Mapping table
CREATE TABLE IF NOT EXISTS plan_events_mapping (
    id SERIAL PRIMARY KEY,
    plan_id INTEGER NOT NULL REFERENCES plans(id),
    event_id INTEGER NOT NULL REFERENCES events(id),
    success_state_emails VARCHAR(225),
    failure_state_emails VARCHAR(225),
    create_date TIMESTAMP WITH TIME ZONE,
    copy_to_customer BOOLEAN DEFAULT false
);

-- Offer Attributes table
CREATE TABLE IF NOT EXISTS offer_attributes (
    id SERIAL PRIMARY KEY,
    offer_id INTEGER NOT NULL REFERENCES offers(id),
    parameter_id VARCHAR(225),
    display_name VARCHAR(225),
    description VARCHAR(225),
    type VARCHAR(225),
    values_list TEXT,
    create_date TIMESTAMP WITH TIME ZONE
);

-- Plan Attribute Mapping table
CREATE TABLE IF NOT EXISTS plan_attribute_mapping (
    plan_attribute_id SERIAL PRIMARY KEY,
    plan_id INTEGER NOT NULL REFERENCES plans(id),
    offer_attribute_id INTEGER NOT NULL REFERENCES offer_attributes(id),
    create_date TIMESTAMP WITH TIME ZONE
);

-- Subscription Attribute Values table
CREATE TABLE IF NOT EXISTS subscription_attribute_values (
    id SERIAL PRIMARY KEY,
    subscription_id INTEGER NOT NULL REFERENCES subscriptions(id),
    offer_id INTEGER NOT NULL REFERENCES offers(id),
    plan_id INTEGER NOT NULL REFERENCES plans(id),
    offer_attribute_id INTEGER NOT NULL REFERENCES offer_attributes(id),
    value VARCHAR(225),
    create_date TIMESTAMP WITH TIME ZONE
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_amp_subscription_id ON subscriptions(amp_subscription_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON subscriptions(subscription_status);
CREATE INDEX IF NOT EXISTS idx_plans_offer_id ON plans(offer_id);
CREATE INDEX IF NOT EXISTS idx_metered_dimensions_plan_id ON metered_dimensions(plan_id);
CREATE INDEX IF NOT EXISTS idx_metered_audit_logs_subscription_id ON metered_audit_logs(subscription_id);
CREATE INDEX IF NOT EXISTS idx_metered_plan_scheduler_subscription_id ON metered_plan_scheduler_management(subscription_id);
CREATE INDEX IF NOT EXISTS idx_metered_plan_scheduler_frequency_id ON metered_plan_scheduler_management(frequency_id);
CREATE INDEX IF NOT EXISTS idx_plan_events_mapping_plan_id ON plan_events_mapping(plan_id);

