build:
    cargo build

build-test:
    cargo build -p api_layer_test

test-api-layer:
    XR_ENABLE_API_LAYERS=XR_APILAYER_NOVENDOR_test XR_API_LAYER_PATH=$PWD xrgears

build-test-api-layer:
	just build-test; just test-api-layer
