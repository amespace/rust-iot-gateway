#[cfg(test)]
mod tests {
    use std::time::Duration;

    use moka::future::Cache;

    #[tokio::test]
    async fn ttl_expires() {
        let cache: Cache<String, bool> = Cache::builder()
            .time_to_live(Duration::from_millis(200))
            .build();

        cache.insert("dev-1".into(), true).await;

        assert_eq!(cache.get("dev-1").await, Some(true));
    }
}
