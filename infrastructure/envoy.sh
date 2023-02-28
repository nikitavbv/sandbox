docker run --rm -it \
      -p 9901:9901 \
      -p 443:10000 \
      -v $(pwd)/config.yaml:/envoy-custom.yaml \
      envoyproxy/envoy-dev:932e9e36c5d2416c2f0d768bb5ad4db3284417c6 \
      -c /envoy-custom.yaml

