use anyhow::{Context as _, Result};
use async_graphql::{Context, Object};
use models::{Nft, NftSql, Trait};
use sqlx::{query, query_as, types::Json, PgPool, Postgres, Transaction};
use std::result::Result as StdResult;
use tracing::error;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn nfts(
        &self,
        ctx: &Context<'_>,
        owner: Option<String>,
        r#type: Option<String>,
    ) -> Result<Vec<Nft>> {
        let pool = ctx.data_unchecked::<PgPool>();
        let tokens = get_nfts_db(pool, owner, r#type)
            .await
            .context("Failed to get nfts data from database")?;

        Ok(tokens)
    }

    async fn nft(&self, ctx: &Context<'_>, id: String) -> Result<Nft> {
        let pool = ctx.data_unchecked::<PgPool>();
        let token = get_nft_db(id, pool)
            .await
            .context("Failed to get nft data from database")?;

        Ok(token)
    }
}

#[tracing::instrument(name = "Query nft from database", skip_all)]
async fn get_nfts_db(
    pool: &PgPool,
    owner: Option<String>,
    r#type: Option<String>,
) -> StdResult<Vec<Nft>, sqlx::Error> {
    let ret = query_as!(
        NftSql,
        r#"
        SELECT id, type, owner, url, traits as "traits: Json<Vec<Trait>>", items as "items: Json<Vec<Trait>>", created_at
        FROM nfts
        WHERE ($1::text IS null OR owner = $1)
            AND ($2::text IS null OR type = $2)
        "#,
        owner,
        r#type,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(Into::into)
    .collect();

    Ok(ret)
}

#[tracing::instrument(name = "Query nft from database", skip(pool))]
async fn get_nft_db(id: String, pool: &PgPool) -> StdResult<Nft, sqlx::Error> {
    query_as!(
        NftSql,
        r#"
        SELECT id, type, owner, url, traits as "traits: Json<Vec<Trait>>, items as "items: Json<Vec<Trait>>", created_at FROM nfts WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
    .map(Into::into)
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn insert_nft(&self, ctx: &Context<'_>, nft: Nft) -> Result<Nft> {
        let nft = nft.into();
        let pool = ctx.data_unchecked::<PgPool>();
        let mut tx = pool
            .begin()
            .await
            .context("Failed to start SQL transaction")?;
        insert_nft_db(&nft, &mut tx)
            .await
            .context("Failed to insert the nft into database")?;
        tx.commit()
            .await
            .context("Failed to commit SQL transaction to store new nft")?;

        Ok(nft.into())
    }
}

#[tracing::instrument(name = "Insert nft to database", skip(tx))]
async fn insert_nft_db(
    NftSql {
        id,
        r#type,
        owner,
        url,
        traits,
        items,
        created_at,
    }: &NftSql,
    tx: &mut Transaction<'_, Postgres>,
) -> StdResult<(), sqlx::Error> {
    query!(
        r#"
        INSERT INTO nfts (id, type, owner, url, traits, created_at, items)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT DO NOTHING 
        "#,
        id,
        r#type,
        owner,
        url,
        traits as _,
        created_at,
        items as _,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to insert nft to database: {e:?}");
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "Update nft in database", skip(tx))]
async fn update_nft_db(
    NftSql {
        id,
        r#type,
        owner,
        url,
        traits,
        items,
        created_at,
    }: &NftSql,
    tx: &mut Transaction<'_, Postgres>,
) -> StdResult<(), sqlx::Error> {
    query!(
        r#"
        UPDATE nfts
        SET type = $2, owner = $3, url = $4, traits = $5, items = $6
        WHERE id = $1
        "#,
        id,
        r#type,
        owner,
        url,
        traits as _,
        items as _,
    )
    .execute(&mut *tx)
    .await?;

    Ok(())
}

#[tracing::instrument(name = "Delete nft from database", skip(tx))]
async fn delete_nft_db(
    id: String,
    tx: &mut Transaction<'_, Postgres>,
) -> StdResult<(), sqlx::Error> {
    query!("DELETE FROM nfts WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    Ok(())
}
