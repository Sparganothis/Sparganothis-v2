use protocol::user_identity::NodeIdentity;

pub async fn db_add_guest_login(from: NodeIdentity) -> anyhow::Result<()> {
    tracing::info!("TODO: CRACKHOUSE INSERT for user = {:?}", from);
    Ok(())
}