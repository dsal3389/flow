use async_trait::async_trait;

mod spawn;

pub use spawn::Spawn;

/// a trait that is implemented on types that can be used
/// as handlers for key combo
#[async_trait]
pub trait ComboHandler: Send + Sync {
    fn handler_name(&self) -> &str;

    /// the handler will be called by the combo executer
    /// to execute the handler logic
    async fn handle(&self) -> anyhow::Result<()>;
}
