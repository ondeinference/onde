Pod::Spec.new do |s|
  s.name             = 'onde_inference'
  s.version          = '0.1.0'
  s.summary          = 'On-device LLM inference SDK for Flutter (macOS).'
  s.description      = 'Runs Qwen 2.5 models locally on macOS with Metal acceleration.'
  s.homepage         = 'https://ondeinference.com'
  s.license          = { :type => 'MIT', :file => '../LICENSE' }
  s.author           = { 'Onde Inference' => 'hello@ondeinference.com' }
  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.dependency 'FlutterMacOS'
  s.platform = :osx, '10.15'
  s.pod_target_xcconfig = { 'DEFINES_MODULE' => 'YES' }
  s.swift_version = '5.0'

  s.script_phases = [
    {
      :name => 'Build Rust bridge (onde_inference_dart)',
      :script => <<~SHELL,
        set -e
        RUST_DIR="${PODS_TARGET_SRCROOT}/../rust"
        cd "$RUST_DIR"
        ARCH=$(uname -m)
        if [[ "$ARCH" == "arm64" ]]; then
          TARGET="aarch64-apple-darwin"
        else
          TARGET="x86_64-apple-darwin"
        fi
        cargo build --release --target "$TARGET"
        cp "target/$TARGET/release/libonde_inference_dart.a" \
           "${PODS_TARGET_SRCROOT}/libonde_inference_dart.a"
      SHELL
      :execution_position => :before_compile,
      :output_files => ["${PODS_TARGET_SRCROOT}/libonde_inference_dart.a"],
    }
  ]

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'OTHER_LDFLAGS'  => '-force_load ${PODS_TARGET_SRCROOT}/libonde_inference_dart.a',
  }
  s.preserve_paths = '../rust/**/*'
end
