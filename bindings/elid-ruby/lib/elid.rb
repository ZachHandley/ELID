# frozen_string_literal: true

require_relative "elid/version"
require_relative "elid_ffi"

# ELID - Embedding Locality-Preserving IDs
#
# This module provides Ruby bindings for ELID, a library for generating
# locality-preserving identifiers from high-dimensional embeddings.
#
# The core functionality is provided through UniFFI-generated bindings
# that wrap the Rust implementation.
#
# @example Encoding a simple embedding
#   embedding = Array.new(128) { rand }
#   elid = Elid.encode(embedding, Elid::Profile::MINI128)
#   puts elid  # => "0A1B2C3D..."
#
# @example Batch encoding
#   embeddings = Array.new(100) { Array.new(128) { rand } }
#   elids = Elid.encode_batch(embeddings, Elid::Profile::MINI128)
#
# @example Computing Hamming distance
#   distance = Elid.hamming_distance(elid1, elid2)
#   puts "Distance: #{distance} bits"
module Elid
  # Encoding profiles for ELID generation
  module Profile
    # 128-bit SimHash using random projections (Charikar 2002)
    # Best for: Angular distance preservation, cosine similarity
    # Output: 26-character base32hex string
    MINI128 = ElidFfi::UniProfile::MINI128

    # Morton Z-order curve encoding for 10D quantized coordinates
    # Best for: Fast multi-dimensional indexing
    # Output: 13-character base32hex string
    MORTON10X10 = ElidFfi::UniProfile::MORTON10X10

    # Hilbert space-filling curve for 10D quantized coordinates
    # Best for: Superior locality preservation (5-10% better than Morton)
    # Output: 13-character base32hex string
    HILBERT10X10 = ElidFfi::UniProfile::HILBERT10X10
  end

  # Error raised when ELID operations fail
  class Error < StandardError; end

  class << self
    # Encode a single embedding into an ELID string
    #
    # @param embedding [Array<Float>] The embedding vector (64-2048 dimensions)
    # @param profile [Integer] The encoding profile (use Profile constants)
    # @return [String] Base32hex-encoded ELID (unpadded, lexicographically sortable)
    # @raise [Error] If embedding dimensions are invalid or values are NaN/infinite
    #
    # @example
    #   embedding = Array.new(128) { rand }
    #   elid = Elid.encode(embedding, Elid::Profile::MINI128)
    def encode(embedding, profile)
      ElidFfi.uni_encode(embedding, profile)
    rescue => e
      raise Error, "Encoding failed: #{e.message}"
    end

    # Decode an ELID string back to raw bytes
    #
    # @param elid [String] The ELID string to decode
    # @return [Array<Integer>] Raw byte array
    # @raise [Error] If ELID format is invalid
    #
    # @example
    #   bytes = Elid.decode(elid)
    def decode(elid)
      ElidFfi.uni_decode(elid)
    rescue => e
      raise Error, "Decoding failed: #{e.message}"
    end

    # Compute Hamming distance between two Mini128 ELIDs
    #
    # Returns the number of differing bits (0-128). Lower values indicate
    # more similar embeddings in angular space.
    #
    # @param elid1 [String] First ELID (must be Mini128 profile)
    # @param elid2 [String] Second ELID (must be Mini128 profile)
    # @return [Integer] Hamming distance (0-128)
    # @raise [Error] If ELIDs are not Mini128 profile or formats are invalid
    #
    # @example
    #   distance = Elid.hamming_distance(elid1, elid2)
    #   similarity = 1.0 - (distance / 128.0)
    def hamming_distance(elid1, elid2)
      ElidFfi.uni_hamming_distance(elid1, elid2)
    rescue => e
      raise Error, "Hamming distance failed: #{e.message}"
    end

    # Encode a batch of embeddings in parallel
    #
    # More efficient than encoding individually when processing multiple
    # embeddings. Uses Rayon for parallel processing on the Rust side.
    #
    # @param embeddings [Array<Array<Float>>] Array of embedding vectors
    # @param profile [Integer] The encoding profile (use Profile constants)
    # @return [Array<String>] Array of ELID strings
    # @raise [Error] If any embedding is invalid
    #
    # @example
    #   embeddings = Array.new(1000) { Array.new(128) { rand } }
    #   elids = Elid.encode_batch(embeddings, Elid::Profile::MINI128)
    def encode_batch(embeddings, profile)
      ElidFfi.uni_encode_batch(embeddings, profile)
    rescue => e
      raise Error, "Batch encoding failed: #{e.message}"
    end
  end
end
