use hyper::{Body, Client, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, Instant};

type Error = Box<dyn std::error::Error + Send + Sync>;

struct LoadBalancer {
    backend_servers: Vec<String>,
    health_check_interval: Duration,
    last_health_check: Instant,
    healthy_backends: HashMap<String, bool>,
}

impl LoadBalancer {
    async fn new(backend_servers: Vec<String>, health_check_interval: Duration) -> Self {
        let mut lb = LoadBalancer {
            backend_servers,
            health_check_interval,
            last_health_check: Instant::now(),
            healthy_backends: HashMap::new(),
        };

        lb.health_check().await;

        lb
    }

    async fn health_check(&mut self) {
        let client = Client::new();
        let mut healthy_backends = HashMap::new();

        for backend in &self.backend_servers {
            let uri = format!("http://{}/health", backend);
            let request = Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .unwrap();

            match client.request(request).await {
                Ok(response) => {
                    healthy_backends.insert(backend.clone(), response.status().is_success());
                }
                Err(_) => {
                    healthy_backends.insert(backend.clone(), false);
                }
            }
        }

        self.healthy_backends = healthy_backends;
        self.last_health_check = Instant::now();
    }

    fn choose_backend(&self) -> Option<String> {
        for backend in &self.backend_servers {
            if let Some(&is_healthy) = self.healthy_backends.get(backend) {
                if is_healthy {
                    return Some(backend.clone());
                }
            }
        }
        None
    }
}

async fn handle_request(req: Request<Body>, lb: Arc<RwLock<LoadBalancer>>) -> Result<Response<Body>, Error> {

    print!("{} ", req.method());
    let backend;
    {
        let lb = lb.read().unwrap();
        backend = lb.choose_backend().ok_or("No healthy backend available")?;
    }

    let uri = format!("http://{}", backend);
    let proxied_req = Request::builder()
        .method(req.method())
        .uri(uri)
        .header("Host", req.headers().clone().get("Host").unwrap())
        .header("User-Agent", req.headers().clone().get("User-Agent").unwrap())
        .header("Accept", req.headers().clone().get("Accept").unwrap())
        .header("Accept-Encoding", req.headers().clone().get("Accept-Encoding").unwrap())
        .header("Accept-Language", req.headers().clone().get("Accept-Language").unwrap())
        .header("Connection", req.headers().clone().get("Connection").unwrap())
        .header("Upgrade-Insecure-Requests", req.headers().clone().get("Upgrade-Insecure-Requests").unwrap())
        .body(req.into_body())
        .unwrap();

    let client = Client::new();
    let proxied_response = client.request(proxied_req).await?;

    Ok(proxied_response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let backend_servers = vec!["localhost:3000".to_string(), "localhost:3080".to_string()];
    let health_check_interval = Duration::from_secs(10);

    let lb = LoadBalancer::new(backend_servers.clone(), health_check_interval).await;
    let lb = Arc::new(RwLock::new(lb));

    let make_svc = make_service_fn(|_conn| {
        let lb = lb.clone();
        async { Ok::<_, Error>(service_fn(move |req| handle_request(req, lb.clone()))) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Load balancer listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}