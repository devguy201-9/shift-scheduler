use redis::AsyncCommands;

pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn get(&self, key: &str) -> redis::RedisResult<Option<String>> {
        let mut conn = self.client.get_async_connection().await?;
        conn.get(key).await
    }

    pub async fn set(&self, key: &str, value: &str, ttl_seconds: u64) -> redis::RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        conn.set_ex(key, value, ttl_seconds).await
    }

    pub async fn delete(&self, key: &str) -> redis::RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        conn.del(key).await
    }
}
