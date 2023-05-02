# scylladb

Didn't work on 2gb instance, but 4gb seems to be fine.

Installed using:
```
curl -sSf get.scylladb.com/server | sudo bash
```

After that, run with `scylla`. No systemd service is configured by default, need to do that myself.

`scylla_io_setup` is needed to be run to measure io performance. Results are saved to `/etc/scylla.d/io_properties.yaml` and need to be passed
to the scylla the following way:

```
scylla --io-properties-file /etc/scylla.d/io_properties.yaml
```