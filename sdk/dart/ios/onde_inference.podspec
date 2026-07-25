Pod::Spec.new do |s|
  s.name             = 'onde_inference'
  s.version          = '1.0.2'
  s.summary          = 'On-device LLM inference SDK for Flutter (iOS).'
  s.description      = 'Runs Qwen 2.5 models locally on iOS with Metal acceleration.'
  s.homepage         = 'https://ondeinference.com'
  s.license          = { :type => 'MIT', :file => '../LICENSE' }
  s.author           = { 'Onde Inference' => 'hello@ondeinference.com' }
  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.dependency 'Flutter'
  s.platform = :ios, '15.0'

  s.pod_target_xcconfig = {
    'DEFINES_MODULE'                         => 'YES',
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]'   => 'i386',
  }
  s.swift_version = '5.0'

  s.script_phases = [
    {
      :name => 'Build Rust bridge (onde_inference_dart)',
      :script => <<~SHELL,
        set -e
        # Keep SDKROOT for Apple headers; clear deployment/linker overrides
        # so cargo doesn't inherit incompatible beta toolchain settings.
        unset IPHONEOS_DEPLOYMENT_TARGET MACOSX_DEPLOYMENT_TARGET \
              TVOS_DEPLOYMENT_TARGET LDFLAGS LD RUSTFLAGS
        RUST_DIR="${PODS_TARGET_SRCROOT}/../rust"
        cd "$RUST_DIR"

        if [[ "$PLATFORM_NAME" == "iphonesimulator" ]]; then
          TARGET="aarch64-apple-ios-sim"
        else
          TARGET="aarch64-apple-ios"
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
    'DEFINES_MODULE'                        => 'YES',
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]'  => 'i386',
    'OTHER_LDFLAGS'                         =>
      '-force_load ${PODS_TARGET_SRCROOT}/libonde_inference_dart.a',
  }
  s.preserve_paths = '../rust/**/*'
end
