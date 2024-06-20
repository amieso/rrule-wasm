# rust-recurrence-generator

This project bridges `rust-rrule` to be used from the iOS/Swift app:
- Exposes a `C` API that allows us to call into `Rust` functions
- `Rust` functions call `rust-rrule` in order to get the generate dates (given recurrence rules)
- A pointer to the generated dates is returned
  - Dates are being exposed as ISO strings (to make data compatibility easier across `Rust`/`C`/`Swift`)
- The Swift app copies the `C` strings and uses them to create `Swift.Date`s

⚠️ **It's important that the consumer of this API calls `free_string_array` for `Rust` to free the allocated memory**

### How to build

1. Run `cargo install` to install `Rust`'s dependencies
2. Run `make build_xc_framework` to generate the `.xcframework`

    a. This will be located in `./target/ios/release/rust_recurrence_generator.xcframework`

    b. The output of the command is a static library

⚠️ **If you change the Rust API, make sure to update the `headers/c_api.h` file to reflect the API changes to `C`**

### How to integrate into the iOS app (through SPM)

1. Locate the `.xcframework`

    a. In this case: `./target/ios/release/rust_recurrence_generator.xcframework`
   
3. Copy the `.xcframework` into the Swift Package
4. Add a `binaryTarget` pointing to the copied `.xcframework` to the project's `Package.swift`
   
    a. In this case: `.binaryTarget(name: "rust_recurrence_generator", path: "rust_recurrence_generator.xcframework")`
   
6. Add `rust_recurrence_generator` as a dependency to any module that need to use the library
7. In the module, simply `import rust_recurrence_generator`
   
    a. Then you'll be able to call the functions defined in the `.src/headers/c_api.h` header file

### References

- https://burgers.io/calling-rust-from-ios
- https://krirogn.dev/blog/integrate-rust-in-ios
