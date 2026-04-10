require 'json'

package = JSON.parse(File.read(File.join(__dir__, '..', 'package.json')))

Pod::Spec.new do |s|
  s.name           = 'OndeInference'
  s.version        = package['version']
  s.summary        = package['description']
  s.description    = package['description']
  s.license        = package['license']
  s.author         = package['author']
  s.homepage       = 'https://ondeinference.com'
  s.platforms      = { :ios => '16.0' }
  s.source         = { git: 'https://github.com/ondeinference/onde.git', tag: s.version.to_s }
  s.static_framework = true

  s.dependency 'ExpoModulesCore'

  s.source_files = '**/*.{h,m,mm,swift,hpp,cpp}'

  # XCFramework bundles both device (aarch64-apple-ios) and simulator
  # (aarch64-apple-ios-sim) slices. CocoaPods selects the right one
  # automatically based on the target SDK.
  s.vendored_frameworks = 'rust/OndeReactNative.xcframework'

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'SWIFT_COMPILATION_MODE' => 'wholemodule',
    'OTHER_LDFLAGS' => '-londe_react_native',
  }
end
