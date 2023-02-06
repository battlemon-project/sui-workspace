use models::{Nft, Trait};

pub mod schema {
    cynic::use_schema!("schema.graphql");
}

type DateTime = chrono::DateTime<chrono::Utc>;
cynic::impl_scalar!(DateTime, schema::DateTime);

#[cynic::schema_for_derives(file = "schema.graphql")]
pub mod insert_nft {
    use super::schema;
    use super::DateTime;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        variables = "InsertNftMutationArguments",
        graphql_type = "MutationRoot"
    )]
    pub struct InsertNftMutation {
        #[arguments(nft : {
        id: $id,
        type: $r#type,
        owner: $owner,
        url: $url,
        createdAt: $created_at,
        traits: $traits,
        items: $items,
        })]
        pub insert_nft: bool,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct InsertNftMutationArguments {
        pub id: String,
        #[cynic(rename = "type")]
        pub r#type: String,
        pub owner: String,
        pub url: String,
        pub traits: Vec<TraitInput>,
        pub items: Vec<NftInput>,
        pub created_at: DateTime,
    }

    #[derive(cynic::InputObject, Debug)]
    pub struct NftInput {
        pub id: String,
        #[cynic(rename = "type")]
        pub r#type: String,
        pub owner: String,
        pub url: String,
        pub traits: Vec<TraitInput>,
        pub items: Vec<NftInput>,
        pub created_at: DateTime,
    }

    #[derive(cynic::InputObject, Debug)]
    pub struct TraitInput {
        pub name: String,
        pub flavour: String,
    }
}

#[cynic::schema_for_derives(file = "schema.graphql")]
pub mod add_item {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(variables = "AddItemMutationArguments", graphql_type = "MutationRoot")]
    pub struct AddItemMutation {
        #[arguments(lemonId: $lemon_id, itemId: $item_id)]
        pub add_item: bool,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct AddItemMutationArguments {
        pub lemon_id: String,
        pub item_id: String,
    }
}

#[cynic::schema_for_derives(file = "schema.graphql")]
pub mod remove_item {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        variables = "RemoveItemMutationArguments",
        graphql_type = "MutationRoot"
    )]
    pub struct RemoveItemMutation {
        #[arguments(lemonId: $lemon_id, itemId: $item_id)]
        pub remove_item: bool,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct RemoveItemMutationArguments {
        pub lemon_id: String,
        pub item_id: String,
    }
}
impl From<Trait> for insert_nft::TraitInput {
    fn from(Trait { name, flavour }: Trait) -> Self {
        Self { name, flavour }
    }
}

impl From<Nft> for insert_nft::NftInput {
    fn from(
        Nft {
            id,
            r#type,
            owner,
            url,
            traits,
            items,
            created_at,
        }: Nft,
    ) -> Self {
        Self {
            id,
            r#type,
            owner,
            url,
            traits: traits.into_iter().map(Into::into).collect(),
            items: items.into_iter().map(Into::into).collect(),
            created_at,
        }
    }
}
