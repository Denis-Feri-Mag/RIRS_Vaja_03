use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const UP: &str = r#"
CREATE TYPE user_role AS ENUM ('CUSTOMER', 'PROVIDER', 'ADMIN');
CREATE TYPE verification_status AS ENUM ('PENDING', 'VERIFIED', 'REJECTED');
CREATE TYPE job_status AS ENUM ('OPEN', 'QUOTED', 'ACCEPTED', 'IN_PROGRESS', 'COMPLETED', 'REVIEWED', 'CANCELLED', 'EXPIRED');
CREATE TYPE quote_status AS ENUM ('PENDING', 'ACCEPTED', 'REJECTED', 'WITHDRAWN');

CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(120) NOT NULL,
    phone VARCHAR(30),
    role user_role NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE provider_profiles (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE RESTRICT,
    business_name VARCHAR(150) NOT NULL,
    tax_number VARCHAR(20) NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    hourly_rate NUMERIC(10,2),
    service_city VARCHAR(100) NOT NULL DEFAULT '',
    service_postal_codes TEXT[] NOT NULL DEFAULT '{}',
    verification_status verification_status NOT NULL DEFAULT 'PENDING',
    verified_by UUID REFERENCES users(id) ON DELETE RESTRICT,
    verified_at TIMESTAMPTZ,
    avg_rating NUMERIC(3,2) NOT NULL DEFAULT 0,
    completed_jobs INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE service_categories (
    id UUID PRIMARY KEY,
    name VARCHAR(80) NOT NULL UNIQUE,
    slug VARCHAR(80) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE provider_categories (
    provider_profile_id UUID NOT NULL REFERENCES provider_profiles(id) ON DELETE RESTRICT,
    category_id UUID NOT NULL REFERENCES service_categories(id) ON DELETE RESTRICT,
    PRIMARY KEY (provider_profile_id, category_id)
);

CREATE TABLE job_requests (
    id UUID PRIMARY KEY,
    customer_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    category_id UUID NOT NULL REFERENCES service_categories(id) ON DELETE RESTRICT,
    title VARCHAR(150) NOT NULL,
    description TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    postal_code VARCHAR(10) NOT NULL,
    address_line VARCHAR(200),
    preferred_from DATE,
    preferred_to DATE,
    budget_min NUMERIC(10,2),
    budget_max NUMERIC(10,2),
    status job_status NOT NULL,
    accepted_quote_id UUID,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE quotes (
    id UUID PRIMARY KEY,
    job_request_id UUID NOT NULL REFERENCES job_requests(id) ON DELETE RESTRICT,
    provider_id UUID NOT NULL REFERENCES provider_profiles(id) ON DELETE RESTRICT,
    price NUMERIC(10,2) NOT NULL CHECK (price > 0),
    estimated_hours NUMERIC(5,1) NOT NULL,
    earliest_start DATE NOT NULL,
    message TEXT,
    status quote_status NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (job_request_id, provider_id)
);

ALTER TABLE job_requests ADD CONSTRAINT job_requests_accepted_quote_fk
    FOREIGN KEY (accepted_quote_id) REFERENCES quotes(id) ON DELETE RESTRICT;

CREATE UNIQUE INDEX quotes_one_accepted ON quotes(job_request_id) WHERE status = 'ACCEPTED';
CREATE INDEX job_requests_matching ON job_requests(status, category_id, postal_code);

CREATE TABLE reviews (
    id UUID PRIMARY KEY,
    job_request_id UUID NOT NULL UNIQUE REFERENCES job_requests(id) ON DELETE RESTRICT,
    customer_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    provider_id UUID NOT NULL REFERENCES provider_profiles(id) ON DELETE RESTRICT,
    rating SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment TEXT,
    provider_reply TEXT,
    is_hidden BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE job_status_history (
    id BIGSERIAL PRIMARY KEY,
    job_request_id UUID NOT NULL REFERENCES job_requests(id) ON DELETE RESTRICT,
    from_status job_status,
    to_status job_status NOT NULL,
    changed_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    note TEXT,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO service_categories (id, name, slug, is_active) VALUES
    (gen_random_uuid(), 'Vodovodna dela', 'vodovodna-dela', TRUE),
    (gen_random_uuid(), 'Elektrika', 'elektrika', TRUE),
    (gen_random_uuid(), 'Pleskanje', 'pleskanje', TRUE),
    (gen_random_uuid(), 'Hišniška dela', 'hisniska-dela', TRUE),
    (gen_random_uuid(), 'Čiščenje', 'ciscenje', TRUE);
"#;

const DOWN: &str = r#"
DROP TABLE job_status_history;
DROP TABLE reviews;
ALTER TABLE job_requests DROP CONSTRAINT job_requests_accepted_quote_fk;
DROP TABLE quotes;
DROP TABLE job_requests;
DROP TABLE provider_categories;
DROP TABLE service_categories;
DROP TABLE provider_profiles;
DROP TABLE users;
DROP TYPE quote_status;
DROP TYPE job_status;
DROP TYPE verification_status;
DROP TYPE user_role;
"#;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(UP).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(DOWN).await?;
        Ok(())
    }
}
