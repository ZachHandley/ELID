# frozen_string_literal: true

require_relative "lib/elid/version"

Gem::Specification.new do |spec|
  spec.name = "elid"
  spec.version = Elid::VERSION
  spec.authors = ["Zach Handley"]
  spec.email = ["zachhandley@gmail.com"]

  spec.summary = "Embedding Locality-Preserving IDs (ELID) for vector databases"
  spec.description = <<~DESC
    ELID provides locality-preserving identifiers for high-dimensional embeddings.
    Supports SimHash-128, Morton curve, and Hilbert curve encoding profiles.
    Built with Rust via UniFFI for performance and safety.
  DESC
  spec.homepage = "https://github.com/zachhandley/ELID"
  spec.licenses = ["MIT", "Apache-2.0"]
  spec.required_ruby_version = ">= 3.0.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/zachhandley/ELID"
  spec.metadata["changelog_uri"] = "https://github.com/zachhandley/ELID/blob/main/CHANGELOG.md"

  # Specify which files should be added to the gem when it is released.
  spec.files = Dir.glob([
    "lib/**/*.rb",
    "lib/**/*.so",
    "lib/**/*.dylib",
    "lib/**/*.dll",
    "README.md",
    "LICENSE-MIT",
    "LICENSE-APACHE"
  ])

  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  # Runtime dependencies
  spec.add_dependency "ffi", "~> 1.15"

  # Development dependencies
  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rspec", "~> 3.0"
  spec.add_development_dependency "rubocop", "~> 1.21"
end
