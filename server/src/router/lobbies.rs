#[tracing::instrument(skip(upgrade))]
pub async fn run(upgrade: fastwebsockets::upgrade::UpgradeFut) -> anyhow::Result<()> {
    let ws = upgrade.await?;
    todo!()
}
