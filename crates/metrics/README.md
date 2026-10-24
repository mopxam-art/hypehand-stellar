# hypehand-metrics

Prometheus metrics registry and telemetry exporter for HypeHand contract invocations and API services.

## Features

- Counter metrics for contract methods (`create_account`, `deposit`, `get_balance`)
- Latency histogram for contract invocation execution durations
- Gauge for active registered user accounts
- Error tracking by method and error code
- OpenMetrics / Prometheus plaintext exporter
