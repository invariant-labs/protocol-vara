use gear_wasm_builder::WasmBuilder;
use gmeta::Metadata;
use io::InvariantMetadata;

fn main() {
    WasmBuilder::with_meta(InvariantMetadata::repr())
        .exclude_features(vec!["binary-vendor"])
        .build();
}
