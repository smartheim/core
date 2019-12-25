# Data History / Runtime Metrics

Collecting runtime metrics is an important factor to evaluate stability and detect unusual service behaviour.
OHX uses the [Influx Database](https://www.influxdata.com/).

This chapter introduces into OHX predefined metrics and how to use the API.
Note however that InfluxDB is purely optional and if it is not running, it will not be used.

## About InfluxDB

InfluxDB is a time-series database that can handle millions of data points per second and is able to compact data to minimize storage space.

InfluxDB uses *Retention Policies* to automate downsampling and data expiration processes.
Downsampling means keeping high-precision raw data for a limited time and storing the lower-precision, summarized data for much longer.

Prometheus metric scraping is integrated, which makes InfluxDB compatible to all services that export in the Prometheus format like [etcd](https://github.com/coreos/etcd/blob/master/Documentation/metrics.md) and Kubernetes.
Services that are interesting in the area of home automation might not export in Prometheus format yet, but chances are high that a *Prometheus Exporter* has been developed like for example for the MQTT Broker Mosquitto: [Mosquitto Exporter](https://github.com/sapcc/mosquitto-exporter).

OHX encourages Addon developers to use the `libohxaddon` library and core service developers to use `libohxcore` for metrics data pushing.
OHX libraries communicate with InfluxDB natively.

A web interface and web APIs are integrated into InfluxDB.
It uses the concept of "Dashboards" for grouping and visualizing multiple metrics together.

## Runtime metrics

Core services have access tokens for InfluxDB and can push metrics as well as query data.
The library `libohxcore` allows to easily post metrics to the database.

The IAM service will configure InfluxDB on startup, and syncs OHX users to InfluxDB.

## Thing states
 
OHX also stores Thing time-series data (eg the Thing state over time) for opt-in Things in the same database.
