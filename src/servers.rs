pub mod servers {
    use std::collections::HashMap;
    use std::convert::Infallible;
    use hyper::{Body, Request, Response, Server};
    use hyper::service::{make_service_fn, service_fn};
    async fn handle_request(req: Request<Body>, server_name: String) -> Result<Response<Body>, Infallible> {
        let mut servers: HashMap<&str, &str> = HashMap::new();
        servers.insert("127.0.0.1:8080", "1");
        servers.insert("127.0.0.1:8081", "2");
        servers.insert("127.0.0.1:8082", "3");
        Ok(Response::new(Body::from(format!("Hello, from server {}", servers.get(server_name.as_str()).unwrap()))))
    }

    pub async fn create_servers() {
        let make_svc = make_service_fn(|_conn| {
            async {
                Ok::<_, Infallible>(service_fn(|req| {
                    let server_name = match req.headers().get("host") {
                        Some(header_value) => header_value.to_str().unwrap_or("Unknown").to_owned(),
                        None => "Unknown".to_owned(),
                    };
                    handle_request(req, server_name.clone())
                }))
            }
        });

        let addr1 = ([127, 0, 0, 1], 8080).into();
        let addr2 = ([127, 0, 0, 1], 8081).into();
        let addr3 = ([127, 0, 0, 1], 8082).into();

        let server1 = Server::bind(&addr1).serve(make_svc.clone());
        let server2 = Server::bind(&addr2).serve(make_svc.clone());
        let server3 = Server::bind(&addr3).serve(make_svc);

        tokio::spawn(server1);
        tokio::spawn(server2);
        tokio::spawn(server3);

        tokio::signal::ctrl_c().await.unwrap();
    }
}