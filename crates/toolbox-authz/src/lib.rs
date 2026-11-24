//! Authentication and authorization (docs/v1-spec.md §9; authorization.md §4.3, D38).
//!
//! - [`Authorizer`]: the contract (G3); [`openfga::OpenFga`] is the v1 implementation, run as a
//!   co-process inside the container.
//! - [`fga_dsl`]: OpenFGA modelling language → model JSON.
//! - [`fgatest`]: `.fga.yaml` tests through the Authorizer.
//! - [`local`]: checks from tuples alone, for a checkout with no server.
//! - [`authn`]: identity from an identity-aware proxy, OIDC bearer tokens, PATs or dev headers.

pub mod authn;
pub mod fga_dsl;
pub mod fgatest;
pub mod local;
pub mod openfga;

pub use openfga::{Datastore, OpenFga, Tuple};

/// The authorization contract (docs/v1-spec.md §7.1).
#[async_trait::async_trait]
pub trait Authorizer: Send + Sync {
    async fn check(&self, user: &str, relation: &str, object: &str) -> Result<bool, String>;
    async fn batch_check(&self, checks: &[Tuple]) -> Result<Vec<bool>, String>;
    async fn list_objects(
        &self,
        user: &str,
        relation: &str,
        object_type: &str,
    ) -> Result<Vec<String>, String>;
    /// The model's JSON form (to know which types and relations exist).
    fn model(&self) -> &serde_json::Value;
    /// Apply a tuple delta, for implementations that hold tuples themselves (in-memory mode).
    /// The Postgres view implementation reads `toolbox.derived_tuples` and ignores this.
    async fn sync_tuples(&self, added: &[Tuple], removed: &[Tuple]) -> Result<(), String>;
}

#[async_trait::async_trait]
impl Authorizer for OpenFga {
    async fn check(&self, user: &str, relation: &str, object: &str) -> Result<bool, String> {
        OpenFga::check(self, user, relation, object).await
    }
    async fn batch_check(&self, checks: &[Tuple]) -> Result<Vec<bool>, String> {
        OpenFga::batch_check(self, checks).await
    }
    async fn list_objects(
        &self,
        user: &str,
        relation: &str,
        object_type: &str,
    ) -> Result<Vec<String>, String> {
        OpenFga::list_objects(self, user, relation, object_type).await
    }
    fn model(&self) -> &serde_json::Value {
        &self.model
    }
    async fn sync_tuples(&self, added: &[Tuple], removed: &[Tuple]) -> Result<(), String> {
        match self.datastore {
            Datastore::Memory => self.write_tuples(added, removed).await,
            Datastore::Postgres(_) => Ok(()),
        }
    }
}
