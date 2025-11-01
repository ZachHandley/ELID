Pod::Spec.new do |s|
  s.name             = 'elid'
  s.version          = '0.1.0'
  s.summary          = 'Flutter bindings for ELID (Embedding Locality-preserving IDentifiers)'
  s.description      = <<-DESC
ELID provides efficient encoding and decoding of embedding vectors into sortable string identifiers.
                       DESC
  s.homepage         = 'https://github.com/zachhandley/ELID'
  s.license          = { :file => '../LICENSE-MIT' }
  s.author           = { 'Zach Handley' => 'zachhandley@gmail.com' }
  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.public_header_files = 'Classes/**/*.h'
  s.dependency 'FlutterMacOS'
  s.platform = :osx, '10.14'
  s.pod_target_xcconfig = { 'DEFINES_MODULE' => 'YES' }
  s.swift_version = '5.0'

  # Use prebuilt static library
  s.vendored_libraries = "**/*.a"
end
