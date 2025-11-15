# Lecture 05: Error Handling and Building Resilient Services

## Introduction

Production async services must handle failures gracefully. This lecture covers error handling patterns, retries, timeouts, circuit breakers, and building fault-tolerant systems.

**Duration:** 60 minutes

## Error Handling Fundamentals

### Result Types in Async Functions

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum ServiceError {
    NetworkError(String),
    Timeout,
    InvalidResponse,
    NotFound,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServiceError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ServiceError::Timeout => write!(f, "Request timed out"),
            ServiceError::InvalidResponse => write!(f, "Invalid response"),
            ServiceError::NotFound => write!(f, "Resource not found"),
        }
    }
}

impl Error for ServiceError {}

async fn fetch_data(id: u64) -> Result<String, ServiceError> {
    // Simulated network request
    if id == 0 {
        return Err(ServiceError::NotFound);
    }

    Ok(format!("Data for ID {}", id))
}
```

### Using thiserror for Custom Errors

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum ApiError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Timeout after {0:?}")]
    Timeout(std::time::Duration),

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Invalid input: {0}")]
    Validation(String),
}

async fn get_user(id: u64) -> Result<User, ApiError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;  // Automatically converts sqlx::Error to ApiError

    Ok(user)
}
```

## Retry Patterns

### Simple Retry with Exponential Backoff

```rust
use tokio::time::{sleep, Duration};

async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: usize,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt > max_retries {
                    eprintln!("All {} retries failed", max_retries);
                    return Err(e);
                }

                let delay = Duration::from_millis(100 * 2_u64.pow(attempt as u32 - 1));
                eprintln!("Attempt {} failed: {:?}. Retrying in {:?}", attempt, e, delay);
                sleep(delay).await;
            }
        }
    }
}

// Usage
async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let result = retry_with_backoff(
        || async { fetch_from_unreliable_api().await },
        3  // max retries
    ).await?;

    Ok(())
}
```

### Retry with Jitter

```rust
use rand::Rng;

async fn retry_with_jitter<F, Fut, T, E>(
    mut operation: F,
    max_retries: usize,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0;
    let mut rng = rand::thread_rng();

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt > max_retries {
                    return Err(e);
                }

                // Exponential backoff with jitter
                let base_delay = 100 * 2_u64.pow(attempt as u32 - 1);
                let jitter = rng.gen_range(0..base_delay / 2);
                let delay = Duration::from_millis(base_delay + jitter);

                eprintln!("Retry {} after {:?}", attempt, delay);
                sleep(delay).await;
            }
        }
    }
}
```

## Circuit Breaker Pattern

Prevent cascading failures by stopping requests to failing services:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Instant, Duration};

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,      // Normal operation
    Open,        // Blocking requests
    HalfOpen,    // Testing if service recovered
}

struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<usize>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    fn new(threshold: usize, timeout: Duration) -> Self {
        CircuitBreaker {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            threshold,
            timeout,
        }
    }

    async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        // Check circuit state
        let state = *self.state.read().await;

        match state {
            CircuitState::Open => {
                // Check if timeout expired
                let last_failure = self.last_failure_time.read().await;
                if let Some(time) = *last_failure {
                    if time.elapsed() > self.timeout {
                        // Try half-open
                        *self.state.write().await = CircuitState::HalfOpen;
                    } else {
                        return Err(CircuitBreakerError::CircuitOpen);
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Allow one request through to test
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(CircuitBreakerError::OperationFailed(e))
            }
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;
        if *state == CircuitState::HalfOpen {
            *state = CircuitState::Closed;
            *self.failure_count.write().await = 0;
        }
    }

    async fn on_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;

        if *failure_count >= self.threshold {
            *self.state.write().await = CircuitState::Open;
            *self.last_failure_time.write().await = Some(Instant::now());
            eprintln!("Circuit breaker opened after {} failures", failure_count);
        }
    }
}

#[derive(Debug)]
enum CircuitBreakerError<E> {
    CircuitOpen,
    OperationFailed(E),
}

// Usage
async fn example_circuit_breaker() -> Result<(), Box<dyn std::error::Error>> {
    let breaker = CircuitBreaker::new(3, Duration::from_secs(30));

    for i in 0..10 {
        match breaker.call(|| async { unreliable_service().await }).await {
            Ok(result) => println!("Success: {}", result),
            Err(CircuitBreakerError::CircuitOpen) => {
                println!("Circuit is open, request blocked");
            }
            Err(CircuitBreakerError::OperationFailed(e)) => {
                println!("Operation failed: {:?}", e);
            }
        }

        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
```

## Timeout Patterns

### Per-Request Timeout

```rust
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout(url: &str) -> Result<String, ApiError> {
    match timeout(Duration::from_secs(5), fetch_data(url)).await {
        Ok(Ok(data)) => Ok(data),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(ApiError::Timeout(Duration::from_secs(5))),
    }
}
```

### Global Timeout with Cleanup

```rust
use tokio::time::timeout;
use tokio::select;

async fn operation_with_cleanup() -> Result<(), ApiError> {
    let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();

    let work = async {
        select! {
            result = do_work() => result,
            _ = cancel_rx => {
                cleanup().await;
                Err(ApiError::Timeout(Duration::from_secs(10)))
            }
        }
    };

    match timeout(Duration::from_secs(10), work).await {
        Ok(result) => result,
        Err(_) => {
            let _ = cancel_tx.send(());
            Err(ApiError::Timeout(Duration::from_secs(10)))
        }
    }
}
```

## Bulkhead Pattern

Isolate resources to prevent cascading failures:

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

struct Bulkhead {
    semaphore: Arc<Semaphore>,
}

impl Bulkhead {
    fn new(max_concurrent: usize) -> Self {
        Bulkhead {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, String>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        let permit = self.semaphore
            .acquire()
            .await
            .map_err(|_| "Failed to acquire permit")?;

        let result = operation().await;

        drop(permit);  // Release permit
        Ok(result)
    }
}

// Usage: Separate bulkheads for different services
async fn example_bulkheads() {
    let database_bulkhead = Bulkhead::new(20);
    let api_bulkhead = Bulkhead::new(10);

    // Database calls limited to 20 concurrent
    database_bulkhead.execute(|| async {
        query_database().await
    }).await.unwrap();

    // API calls limited to 10 concurrent
    api_bulkhead.execute(|| async {
        call_external_api().await
    }).await.unwrap();
}
```

## Health Checks and Monitoring

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct ServiceMetrics {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    total_latency_ms: AtomicU64,
}

impl ServiceMetrics {
    fn new() -> Arc<Self> {
        Arc::new(ServiceMetrics {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
        })
    }

    fn record_request(&self, latency_ms: u64, success: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);

        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn success_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 1.0;
        }

        let successful = self.successful_requests.load(Ordering::Relaxed);
        successful as f64 / total as f64
    }

    fn average_latency(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }

        let latency = self.total_latency_ms.load(Ordering::Relaxed);
        latency as f64 / total as f64
    }
}

// Health check endpoint
async fn health_check(
    State(metrics): State<Arc<ServiceMetrics>>,
) -> (StatusCode, Json<HealthResponse>) {
    let success_rate = metrics.success_rate();
    let avg_latency = metrics.average_latency();

    let healthy = success_rate > 0.95 && avg_latency < 1000.0;

    let status = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = HealthResponse {
        healthy,
        success_rate,
        average_latency_ms: avg_latency,
    };

    (status, Json(response))
}

#[derive(Serialize)]
struct HealthResponse {
    healthy: bool,
    success_rate: f64,
    average_latency_ms: f64,
}
```

## Graceful Degradation

```rust
async fn get_user_with_fallback(id: u64) -> Result<User, ApiError> {
    // Try primary database
    match timeout(
        Duration::from_millis(500),
        fetch_from_primary_db(id)
    ).await {
        Ok(Ok(user)) => return Ok(user),
        Ok(Err(e)) => eprintln!("Primary DB error: {:?}", e),
        Err(_) => eprintln!("Primary DB timeout"),
    }

    // Fallback to cache
    match fetch_from_cache(id).await {
        Ok(user) => {
            eprintln!("Served from cache (degraded)");
            return Ok(user);
        }
        Err(e) => eprintln!("Cache error: {:?}", e),
    }

    // Final fallback: return default
    Ok(User::default_for_id(id))
}
```

## Structured Logging

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(db), fields(user_id = %id))]
async fn get_user(id: u64, db: &Database) -> Result<User, ApiError> {
    info!("Fetching user");

    match db.query_user(id).await {
        Ok(user) => {
            info!(username = %user.name, "User found");
            Ok(user)
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch user");
            Err(ApiError::Database(e))
        }
    }
}

// Initialize tracing
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();
}
```

## Complete Resilient Service Example

```rust
use axum::{Router, routing::get, extract::State};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
struct AppState {
    db: Database,
    metrics: Arc<ServiceMetrics>,
    rate_limiter: Arc<Semaphore>,
    circuit_breaker: CircuitBreaker,
}

async fn get_user_handler(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<User>, AppError> {
    let start = Instant::now();

    // Rate limiting
    let _permit = state.rate_limiter
        .acquire()
        .await
        .map_err(|_| AppError::RateLimited)?;

    // Circuit breaker
    let result = state.circuit_breaker.call(|| async {
        // Retry logic
        retry_with_backoff(
            || fetch_user_with_timeout(id, &state.db),
            3
        ).await
    }).await;

    // Record metrics
    let latency = start.elapsed().as_millis() as u64;
    state.metrics.record_request(latency, result.is_ok());

    match result {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            error!(error = ?e, user_id = id, "Failed to fetch user");
            Err(e.into())
        }
    }
}

#[tokio::main]
async fn main() {
    let state = AppState {
        db: Database::connect().await.unwrap(),
        metrics: ServiceMetrics::new(),
        rate_limiter: Arc::new(Semaphore::new(100)),
        circuit_breaker: CircuitBreaker::new(5, Duration::from_secs(30)),
    };

    let app = Router::new()
        .route("/users/:id", get(get_user_handler))
        .route("/health", get(health_check))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Exercises

1. **Retry Logic**: Implement exponential backoff with jitter for a flaky API
2. **Circuit Breaker**: Build a complete circuit breaker and test failure scenarios
3. **Health Monitoring**: Create endpoints that report service health metrics
4. **Graceful Shutdown**: Implement clean shutdown that drains requests

## Key Takeaways

1. **Always handle errors explicitly** - no unwrap() in production
2. **Retry with exponential backoff** - but add jitter
3. **Circuit breakers prevent cascades** - fail fast when service is down
4. **Timeouts are mandatory** - never wait forever
5. **Monitor everything** - metrics drive reliability

## Next Steps

You've completed Module 02! You now understand:
- Async fundamentals and Tokio runtime
- TCP servers and connection handling
- HTTP services with Axum
- Channels and concurrent patterns
- Error handling and resilience

Next module: **Key-Value Store** - apply these concepts to build a persistent storage engine.

## Resources

- [Tokio Best Practices](https://tokio.rs/tokio/topics/best-practices)
- [AWS Well-Architected Framework](https://aws.amazon.com/architecture/well-architected/)
- ["Release It!" by Michael Nygard](https://pragprog.com/titles/mnee2/release-it-second-edition/)
