# Santulankarta: A 7-Layer Load Balancer Written in Rust

Santulankarta is a simple Layer 7 load balancer implementation written in Rust. This project aims to demonstrate the basic concepts of load balancing and health checking using the Rust programming language and the hyperweb framework.

## Features

- Distributes incoming HTTP requests to backend servers.
- Performs health checks on backend servers to ensure their availability.
- Proxies requests to healthy backend servers for processing.

## Why the name?

Santulankarta is the sankrit term for "Balancer".

## Getting Started

### Prerequisites

- Rust programming language and Cargo (Rust's package manager) must be installed. You can download Rust from the official website: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Installation

1. Clone the repository:

   ```sh
   git clone https://github.com/KunalKatiyar/santulankarta.git
   cd santulankarta
   ```

2. Build the project using Cargo:

   ```sh
   cargo build --release
   ```

3. Run the load balancer:

   ```sh
   cargo run --release
   ```

The load balancer will start on `http://127.0.0.1:8080`.

### Testing

1. Ensure that the backend servers are running and respond to health check requests.

2. Use tools like `curl` to make requests to the load balancer:

   ```sh
   curl -I http://localhost:8080/
   ```

3. Observe the behavior of the load balancer as it routes requests to different backend servers.

## Contributing

Contributions are welcome! If you find any issues or want to enhance the project, feel free to submit pull requests or open issues.

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgment

- This project was inspired by the need to understand the basics of load balancing and Rust programming.
- Thanks to the Rust community for creating such a powerful and safe programming language.
