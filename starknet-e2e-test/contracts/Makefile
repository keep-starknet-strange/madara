.PHONY: artifacts

artifacts:
	mkdir artifacts || true
	forge build
	forge build --root ../../madara-zaun
	cp build/UnsafeProxy.sol/UnsafeProxy.json artifacts/
	cp ../../madara-zaun/out/StarknetValidium.sol/Starknet.json artifacts/