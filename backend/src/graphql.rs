use anyhow::{Context as _, Result};
use async_graphql::{Context, Object};
use models::{NftToken, NftTokenSql, Trait};
use sqlx::{query, query_as, types::Json, PgPool, Postgres, Transaction};
use std::result::Result as StdResult;
use tracing::error;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn nft_tokens(&self, ctx: &Context<'_>, owner: Option<String>) -> Result<Vec<NftToken>> {
        let pool = ctx.data_unchecked::<PgPool>();
        let tokens = get_nft_tokens_db(pool, owner)
            .await
            .context("Failed to get nft tokens data from database")?;

        Ok(tokens)
    }

    async fn nft_token(&self, ctx: &Context<'_>, id: String) -> Result<NftToken> {
        let pool = ctx.data_unchecked::<PgPool>();
        let token = get_nft_token_db(id, pool)
            .await
            .context("Failed to get nft token data from database")?;

        Ok(token)
    }
}

#[tracing::instrument(name = "Query nft tokens from database", skip_all)]
async fn get_nft_tokens_db(
    pool: &PgPool,
    owner: Option<String>,
) -> StdResult<Vec<NftToken>, sqlx::Error> {
    let ret = query_as!(
        NftTokenSql,
        r#"
        SELECT id, type, owner, url, traits as "traits: Json<Vec<Trait>>", created_at
        FROM nft_tokens
        WHERE $1::text IS null OR owner = $1
        "#,
        owner
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(Into::into)
    .collect();

    Ok(ret)
}

#[tracing::instrument(name = "Query nft tokens from database", skip(pool))]
async fn get_nft_token_db(id: String, pool: &PgPool) -> StdResult<NftToken, sqlx::Error> {
    query_as!(
        NftTokenSql,
        r#"
        SELECT id, type, owner, url, traits as "traits: Json<Vec<Trait>>", created_at FROM nft_tokens WHERE id = $1
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
    async fn insert_nft_token(&self, ctx: &Context<'_>, nft_token: NftToken) -> Result<NftToken> {
        let nft_token = nft_token.into();
        let pool = ctx.data_unchecked::<PgPool>();
        let mut tx = pool
            .begin()
            .await
            .context("Failed to start SQL transaction")?;
        insert_nft_token_db(&nft_token, &mut tx)
            .await
            .context("Failed to insert the nft token into database")?;
        tx.commit()
            .await
            .context("Failed to commit SQL transaction to store new nft token")?;

        Ok(nft_token.into())
    }
}

#[tracing::instrument(name = "Insert nft tokens to database", skip(tx))]
async fn insert_nft_token_db(
    nft_token: &NftTokenSql,
    tx: &mut Transaction<'_, Postgres>,
) -> StdResult<(), sqlx::Error> {
    query!(
        r#"
        INSERT INTO nft_tokens (id, type, owner, url, traits, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT DO NOTHING 
        "#,
        nft_token.id,
        nft_token.r#type,
        nft_token.owner,
        nft_token.url,
        nft_token.traits as _,
        nft_token.created_at,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to insert nft token to database: {e:?}");
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "Update nft token in database", skip(tx))]
async fn update_nft_token_db(
    nft_token: NftTokenSql,
    tx: &mut Transaction<'_, Postgres>,
) -> StdResult<(), sqlx::Error> {
    query!(
        r#"
        UPDATE nft_tokens
        SET type = $2, owner = $3, url = $4, traits = $5
        WHERE id = $1
        "#,
        nft_token.id,
        nft_token.r#type,
        nft_token.owner,
        nft_token.url,
        nft_token.traits as _,
    )
    .execute(&mut *tx)
    .await?;

    Ok(())
}

#[tracing::instrument(name = "Delete nft token from database", skip(tx))]
async fn delete_nft_token_db(
    id: String,
    tx: &mut Transaction<'_, Postgres>,
) -> StdResult<(), sqlx::Error> {
    query!("DELETE FROM nft_tokens WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    Ok(())
}
