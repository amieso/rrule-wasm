release:
	cargo build --target aarch64-apple-ios --target aarch64-apple-ios-sim --package rust_recurrence_generator --release

build_xc_framework:
	make release
	rm -rf ./../target/ios/
	xcodebuild -create-xcframework \
	-library "./../target/aarch64-apple-ios/release/librust_recurrence_generator.a" \
	-headers "./src/headers" \
	-library "./../target/aarch64-apple-ios-sim/release/librust_recurrence_generator.a" \
	-headers "./src/headers" \
	-output "./../target/ios/release/rust_recurrence_generator.xcframework"