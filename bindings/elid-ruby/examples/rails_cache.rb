#!/usr/bin/env ruby
# frozen_string_literal: true

# Rails Cache Integration Example
#
# This demonstrates using ELID as cache keys for embedding-based lookups
# in a Rails application with Redis backend.
#
# Usage:
#   rails console
#   load 'examples/rails_cache.rb'
#   ElidCacheExample.run

require 'elid'

module ElidCacheExample
  # Configuration for ELID-based caching
  class Config
    # Which profile to use for cache keys
    CACHE_PROFILE = Elid::Profile::MINI128

    # Cache TTL in seconds (1 hour)
    CACHE_TTL = 3600

    # Namespace for ELID cache keys
    CACHE_NAMESPACE = "elid:v1"
  end

  # ELID-based cache wrapper for Rails.cache
  class ElidCache
    class << self
      # Store a value with an embedding-based key
      #
      # @param embedding [Array<Float>] The embedding vector
      # @param value [Object] Value to cache (must be serializable)
      # @param ttl [Integer] Time to live in seconds
      # @return [String] The generated ELID key
      def set(embedding, value, ttl: Config::CACHE_TTL)
        elid = generate_key(embedding)
        Rails.cache.write(elid, value, expires_in: ttl)
        Rails.logger.info("ELID Cache SET: #{elid} (TTL: #{ttl}s)")
        elid
      end

      # Retrieve a value by ELID key
      #
      # @param elid [String] The ELID cache key
      # @return [Object, nil] Cached value or nil if not found
      def get(elid)
        value = Rails.cache.read(elid)
        Rails.logger.info("ELID Cache GET: #{elid} -> #{value ? 'HIT' : 'MISS'}")
        value
      end

      # Retrieve a value by embedding (generates ELID first)
      #
      # @param embedding [Array<Float>] The embedding vector
      # @return [Object, nil] Cached value or nil if not found
      def get_by_embedding(embedding)
        elid = generate_key(embedding)
        get(elid)
      end

      # Delete a cached value
      #
      # @param elid [String] The ELID cache key
      # @return [Boolean] True if deleted
      def delete(elid)
        result = Rails.cache.delete(elid)
        Rails.logger.info("ELID Cache DELETE: #{elid}")
        result
      end

      # Find all keys with a given prefix (for approximate search)
      #
      # Note: This requires Redis and is expensive - use sparingly!
      #
      # @param prefix [String] ELID prefix (e.g., first 6 characters)
      # @param limit [Integer] Maximum results
      # @return [Array<String>] Matching ELID keys
      def find_by_prefix(prefix, limit: 100)
        return [] unless redis?

        pattern = "#{Config::CACHE_NAMESPACE}:#{prefix}*"
        Redis.current.scan_each(match: pattern).first(limit)
      end

      # Fetch with automatic caching (like Rails.cache.fetch)
      #
      # @param embedding [Array<Float>] The embedding vector
      # @param ttl [Integer] Time to live in seconds
      # @yield Block to execute if cache miss
      # @return [Object] Cached or computed value
      def fetch(embedding, ttl: Config::CACHE_TTL)
        elid = generate_key(embedding)

        cached = get(elid)
        return cached if cached

        value = yield
        set(embedding, value, ttl: ttl)
        value
      end

      private

      def generate_key(embedding)
        base_elid = Elid.encode(embedding, Config::CACHE_PROFILE)
        "#{Config::CACHE_NAMESPACE}:#{base_elid}"
      end

      def redis?
        defined?(Redis) && Rails.cache.is_a?(ActiveSupport::Cache::RedisCacheStore)
      end
    end
  end

  # Example model using ELID for primary key
  #
  # Migration:
  #   create_table :semantic_documents, id: false do |t|
  #     t.string :id, primary_key: true, limit: 64  # ELID with namespace
  #     t.text :content
  #     t.json :embedding
  #     t.timestamps
  #   end
  #   add_index :semantic_documents, :id, unique: true
  #
  class SemanticDocument
    include ActiveModel::Model

    attr_accessor :id, :content, :embedding, :created_at, :updated_at

    # Generate ELID from embedding before save
    def before_create
      self.id = generate_elid
      self.created_at = Time.current
      self.updated_at = Time.current
    end

    # Find similar documents by ELID prefix
    #
    # This leverages database index on lexicographically sorted ELIDs
    def self.find_similar(elid, limit: 10)
      prefix = elid[0..8]  # First 9 characters for locality
      where("id LIKE ?", "#{prefix}%").limit(limit)
    end

    private

    def generate_elid
      return id if id.present?

      base_elid = Elid.encode(embedding, Elid::Profile::MINI128)
      "doc:#{base_elid}"
    end
  end

  # Similarity search with caching
  class SimilaritySearch
    # Find cached similar items
    #
    # @param query_embedding [Array<Float>] Query vector
    # @param top_k [Integer] Number of results
    # @return [Array<Hash>] Cached results or nil
    def self.cached_search(query_embedding, top_k: 10)
      cache_key = "search:#{Elid.encode(query_embedding, Elid::Profile::MINI128)}"

      ElidCache.fetch(query_embedding, ttl: 300) do
        # Expensive similarity search (e.g., vector database query)
        perform_similarity_search(query_embedding, top_k)
      end
    end

    # Compare two embeddings via Hamming distance
    #
    # @param embedding1 [Array<Float>] First embedding
    # @param embedding2 [Array<Float>] Second embedding
    # @return [Float] Approximate cosine similarity (0.0-1.0)
    def self.approximate_similarity(embedding1, embedding2)
      elid1 = Elid.encode(embedding1, Elid::Profile::MINI128)
      elid2 = Elid.encode(embedding2, Elid::Profile::MINI128)

      distance = Elid.hamming_distance(elid1, elid2)

      # Approximate cosine similarity from Hamming distance
      # For SimHash: cos(θ) ≈ 1 - (hamming_distance / 64)
      1.0 - (distance.to_f / 64.0)
    end

    def self.perform_similarity_search(query_embedding, top_k)
      # Placeholder for actual vector database query
      # In production, this would query pgvector, Pinecone, etc.
      {
        query: query_embedding[0..5],
        results: Array.new(top_k) do |i|
          {
            id: "doc_#{i}",
            score: rand,
            content: "Result #{i}"
          }
        end,
        cached_at: Time.current
      }
    end
  end

  # Demo runner
  def self.run
    puts "\n=== ELID Rails Cache Example ===\n\n"

    # Example 1: Basic caching
    puts "1. Basic Embedding Caching"
    embedding = Array.new(128) { rand }

    elid = ElidCache.set(embedding, { result: "expensive computation" }, ttl: 60)
    puts "   Cached with ELID: #{elid}"

    cached_value = ElidCache.get_by_embedding(embedding)
    puts "   Retrieved: #{cached_value.inspect}"

    # Example 2: Cache fetch pattern
    puts "\n2. Fetch with Automatic Caching"
    result = ElidCache.fetch(embedding, ttl: 60) do
      puts "   Cache MISS - Computing value..."
      { timestamp: Time.current, data: "computed result" }
    end
    puts "   Result: #{result.inspect}"

    result2 = ElidCache.fetch(embedding, ttl: 60) do
      puts "   This shouldn't print (cache HIT)"
      { timestamp: Time.current, data: "new result" }
    end
    puts "   Cached result: #{result2.inspect}"

    # Example 3: Similarity comparison
    puts "\n3. Approximate Similarity via Hamming Distance"
    embedding1 = Array.new(128) { rand }
    embedding2 = embedding1.map { |x| x + rand * 0.1 }  # Similar

    similarity = SimilaritySearch.approximate_similarity(embedding1, embedding2)
    puts "   Similarity (similar embeddings): #{(similarity * 100).round(2)}%"

    embedding3 = Array.new(128) { rand }  # Random
    similarity2 = SimilaritySearch.approximate_similarity(embedding1, embedding3)
    puts "   Similarity (random embeddings): #{(similarity2 * 100).round(2)}%"

    # Example 4: Batch operations
    puts "\n4. Batch Caching"
    embeddings = Array.new(10) { Array.new(128) { rand } }
    elids = Elid.encode_batch(embeddings, Elid::Profile::MINI128)

    embeddings.zip(elids).each_with_index do |(embedding, elid), idx|
      ElidCache.set(embedding, { index: idx, data: "batch item #{idx}" })
    end
    puts "   Cached #{elids.length} items"

    # Example 5: ELID-based primary keys
    puts "\n5. ELID as Primary Key"
    doc = SemanticDocument.new(
      content: "Sample document",
      embedding: Array.new(128) { rand }
    )
    doc.before_create
    puts "   Generated ID: #{doc.id}"
    puts "   ID format: doc:<elid>"

    puts "\n=== Example Complete ===\n"
  end
end

# Run if executed directly
if __FILE__ == $PROGRAM_NAME
  # Simulate Rails environment
  module Rails
    class << self
      def cache
        @cache ||= begin
          require 'active_support'
          require 'active_support/cache'
          ActiveSupport::Cache::MemoryStore.new
        end
      end

      def logger
        @logger ||= Logger.new($stdout)
      end
    end
  end

  ElidCacheExample.run
end
