# Cosmos Upgrade Watcher

Cosmos Upgrade Watcher is a Rust program that monitors the upcoming upgrade status of Cosmos SDK chains. It fetches upgrade information from a specified API endpoint, filters the upgrades based on a watch list, and exposes the upgrade information as Prometheus metrics.

## Features

- Fetches upgrade information from a specified API endpoint.
- Filters upgrades based on a configurable watch list.
- Exposes upgrade information as Prometheus metrics.
- Removes expired metrics based on the estimated upgrade time.

## Configuration

The program uses a configuration file in TOML format. Below is an example configuration file (`config.toml`):

```toml
[prometheus]
host = '127.0.0.1'
port = 9090

[chain]
watch_list = [
  'cosmos',
  'osmosis',
  'stargaze',
  'mantra',
]
refresh = '60s'
endpoint = 'https://polkachu.com/api/v2/chain_upgrades'
```

- `prometheus.host`: The host address for the Prometheus metrics server.
- `prometheus.port`: The port for the Prometheus metrics server.
- `chain.watch_list`: A list of networks to watch for upgrades.
- `chain.refresh`: The interval at which to refresh the upgrade information.
- `chain.endpoint`: The API endpoint to fetch the upgrade information.

## Usage

1. Clone the repository:

```sh
git clone https://github.com/allthatjazzleo/cosmos-upgrade-watcher.git
cd cosmos-upgrade-watcher
```

2. Create a configuration file (`config.toml`) in the root directory with the desired settings.

3. Build and run the program:

```sh
cargo build --release
./target/release/cosmos-upgrade-watcher -c config.toml
```

4. The Prometheus metrics will be available at `http://127.0.0.1:9090/metrics` (or the configured host and port).

## Running Tests

To run the unit tests, use the following command:

```sh
cargo test
```

## License

This project is licensed under the Mozilla Public License. See the [LICENSE](LICENSE) file for details.
