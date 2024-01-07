mod servers;

use hyper::{Body, Client, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};

type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(dead_code)]
struct LoadBalancer {
    backend_servers: Vec<String>,
    health_check_interval: Duration,
    last_health_check: Instant,
    healthy_backends: HashMap<String, bool>,
    next_backend_index: usize
}

impl LoadBalancer {
    async fn new(backend_servers: Vec<String>, health_check_interval: Duration) -> Self {
        let mut lb = LoadBalancer {
            backend_servers,
            health_check_interval,
            last_health_check: Instant::now(),
            healthy_backends: HashMap::new(),
            next_backend_index: 0
        };

        lb.health_check().await;

        lb
    }

    async fn health_check(&mut self) {
        let client = Client::new();
        let mut healthy_backends = HashMap::new();

        for backend in &self.backend_servers {
            let uri = format!("http://{}", backend);
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

    fn choose_backend(&mut self) -> Option<String> {
        // Iterate through the backend servers starting from the next index
        for _ in 0..self.backend_servers.len() {
            let backend = &self.backend_servers[self.next_backend_index];

            if let Some(&is_healthy) = self.healthy_backends.get(backend) {
                if is_healthy {
                    // Update the index for the next iteration
                    self.next_backend_index = (self.next_backend_index + 1) % self.backend_servers.len();
                    return Some(backend.clone());
                }
            }

            // Update the index for the next iteration
            self.next_backend_index = (self.next_backend_index + 1) % self.backend_servers.len();
        }

        None
    }
}

async fn handle_request(req: Request<Body>, lb: Arc<Mutex<LoadBalancer>>) -> Result<Response<Body>, Error> {

    let backend;
    {
        let mut lb = lb.lock().unwrap();
        backend = lb.choose_backend().ok_or("No healthy backend available")?;
    }

    let uri = format!("http://{}", backend);
    let proxied_req = Request::builder()
        .method(req.method())
        .uri(uri)
        .header("Host", backend.clone())
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

    tokio::spawn(async move {
        servers::servers::create_servers().await;
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let backend_servers = vec!["127.0.0.1:8081".to_string(), "127.0.0.1:8082".to_string(), "127.0.0.1:8083".to_string(),];
    let health_check_interval = Duration::from_secs(10);

    let lb = LoadBalancer::new(backend_servers.clone(), health_check_interval).await;
    let lb = Arc::new(Mutex::new(lb));



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