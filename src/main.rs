mod servers;
mod balancer;

#[tokio::main]
async fn main() {

    servers::servers::create_servers().await;
}
