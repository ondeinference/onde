# frozen_string_literal: true

require_relative 'lib/onde/version'

Gem::Specification.new do |spec|
  spec.name = 'onde-inference'
  spec.version = Onde::VERSION
  spec.authors = ['Seto Elkahfi']
  spec.email = ['setoelkahfi@gmail.com']

  spec.summary = 'On-device AI inference for Ruby — powered by Rust.'
  spec.description = 'Ruby bindings for the Onde inference engine. ' \
                     'Run LLMs and speech-to-text locally with automatic ' \
                     'HuggingFace model management, cache inspection, and GPU acceleration.'
  spec.homepage = 'https://ondeinference.com'
  spec.license = 'MIT'
  spec.required_ruby_version = '>= 3.0.0'
  spec.required_rubygems_version = '>= 3.3.11'

  spec.metadata['homepage_uri'] = spec.homepage
  spec.metadata['source_code_uri'] = 'https://github.com/ondeinference/onde'
  spec.metadata['changelog_uri'] = 'https://github.com/ondeinference/onde/blob/main/CHANGELOG.md'
  # Tell rb_sys / RbSys::ExtensionTask the actual Rust crate name so it
  # resolves gem/ext/onde-ruby/Cargo.toml instead of walking up to the
  # parent lib/crates/onde/Cargo.toml (name = "onde").
  spec.metadata['cargo_crate_name'] = 'onde-ruby'

  # Explicit file list — git ls-files does not work reliably in monorepo
  # subdirectories that may be untracked or partially ignored.
  # We exclude target/ directories (Cargo build artifacts), compiled native
  # extensions (.bundle, .so, .dylib, .dll, .o, .a), and tmp/ build staging.
  spec.files = Dir.chdir(__dir__) do
    Dir.glob('{lib,ext}/**/*').select { |f| File.file?(f) }
       .reject { |f| f.include?('/target/') }
       .reject { |f| f.match?(/\.(bundle|so|dylib|dll|o|a|dSYM)$/) }
       .reject { |f| f.start_with?('tmp/') } +
    %w[
      Cargo.toml
      Cargo.lock
      README.md
      rust-toolchain.toml
    ]
  end.select { |f| File.exist?(File.join(__dir__, f)) }

  spec.bindir = 'exe'
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ['lib']
  spec.extensions = ['ext/onde-ruby/Cargo.toml']
end
