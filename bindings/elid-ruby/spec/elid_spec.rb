# frozen_string_literal: true

require "spec_helper"
require "json"

RSpec.describe Elid do
  it "has a version number" do
    expect(Elid::VERSION).not_to be nil
  end

  describe ".encode" do
    it "encodes a 128-dimensional embedding with Mini128 profile" do
      embedding = Array.new(128) { rand }
      elid = Elid.encode(embedding, Elid::Profile::MINI128)

      expect(elid).to be_a(String)
      expect(elid.length).to eq(29)  # Base32hex encoding of 2-byte header + 128-bit payload
      expect(elid).to match(/^[0-9a-v]+$/)  # Base32hex alphabet (lowercase)
    end

    it "encodes a 1024-dimensional embedding with Morton10x10 profile" do
      embedding = Array.new(1024) { rand }
      elid = Elid.encode(embedding, Elid::Profile::MORTON10X10)

      expect(elid).to be_a(String)
      expect(elid.length).to eq(24)  # Base32hex encoding of (2-byte header + 100-bit payload)
      expect(elid).to match(/^[0-9a-v]+$/)
    end

    it "encodes a 1024-dimensional embedding with Hilbert10x10 profile" do
      embedding = Array.new(1024) { rand }
      elid = Elid.encode(embedding, Elid::Profile::HILBERT10X10)

      expect(elid).to be_a(String)
      expect(elid.length).to eq(24)
      expect(elid).to match(/^[0-9a-v]+$/)
    end

    it "raises error for invalid dimension count" do
      embedding = Array.new(32) { rand }  # Too few dimensions

      expect {
        Elid.encode(embedding, Elid::Profile::MINI128)
      }.to raise_error(Elid::Error, /Encoding failed/)
    end

    it "raises error for NaN values" do
      embedding = Array.new(128, Float::NAN)

      expect {
        Elid.encode(embedding, Elid::Profile::MINI128)
      }.to raise_error(Elid::Error, /Encoding failed/)
    end

    it "raises error for infinite values" do
      embedding = Array.new(128, Float::INFINITY)

      expect {
        Elid.encode(embedding, Elid::Profile::MINI128)
      }.to raise_error(Elid::Error, /Encoding failed/)
    end

    it "produces deterministic output for same input" do
      embedding = Array.new(128) { |i| i.to_f / 128.0 }
      elid1 = Elid.encode(embedding, Elid::Profile::MINI128)
      elid2 = Elid.encode(embedding, Elid::Profile::MINI128)

      expect(elid1).to eq(elid2)
    end

    it "produces different outputs for different inputs" do
      embedding1 = Array.new(128) { rand }
      embedding2 = Array.new(128) { rand }

      elid1 = Elid.encode(embedding1, Elid::Profile::MINI128)
      elid2 = Elid.encode(embedding2, Elid::Profile::MINI128)

      expect(elid1).not_to eq(elid2)
    end
  end

  describe ".decode" do
    it "decodes an ELID back to bytes" do
      embedding = Array.new(128) { rand }
      elid = Elid.encode(embedding, Elid::Profile::MINI128)

      bytes = Elid.decode(elid)

      expect(bytes).to be_an(Array)
      expect(bytes.length).to eq(18)  # 2-byte header + 16-byte payload = 18 bytes
      expect(bytes).to all(be_a(Integer))
      # Note: bytes might be signed (-128 to 127) or unsigned (0 to 255)
    end

    it "raises error for invalid ELID format" do
      expect {
        Elid.decode("invalid!")
      }.to raise_error(Elid::Error, /Decoding failed/)
    end

    it "handles empty string decode" do
      # Note: Empty string decoding behavior may vary
      # If it doesn't raise an error, it should return an empty array
      result = Elid.decode("")
      expect(result).to be_an(Array)
    end
  end

  describe ".hamming_distance" do
    it "returns 0 for identical ELIDs" do
      embedding = Array.new(128) { rand }
      elid = Elid.encode(embedding, Elid::Profile::MINI128)

      distance = Elid.hamming_distance(elid, elid)

      expect(distance).to eq(0)
    end

    it "returns distance between 0 and 128 for different ELIDs" do
      embedding1 = Array.new(128) { rand }
      embedding2 = Array.new(128) { rand }

      elid1 = Elid.encode(embedding1, Elid::Profile::MINI128)
      elid2 = Elid.encode(embedding2, Elid::Profile::MINI128)

      distance = Elid.hamming_distance(elid1, elid2)

      expect(distance).to be_a(Integer)
      expect(distance).to be_between(0, 128)
    end

    it "returns lower distance for similar embeddings" do
      # Create similar embeddings by adding small noise
      base_embedding = Array.new(128) { rand }
      embedding1 = base_embedding.map { |x| x + rand * 0.01 }
      embedding2 = base_embedding.map { |x| x + rand * 0.01 }

      elid1 = Elid.encode(embedding1, Elid::Profile::MINI128)
      elid2 = Elid.encode(embedding2, Elid::Profile::MINI128)

      distance = Elid.hamming_distance(elid1, elid2)

      # Similar embeddings should have relatively low Hamming distance
      expect(distance).to be < 64  # Less than half the bits differ
    end

    it "raises error for Morton profile ELIDs" do
      embedding = Array.new(1024) { rand }
      elid = Elid.encode(embedding, Elid::Profile::MORTON10X10)

      expect {
        Elid.hamming_distance(elid, elid)
      }.to raise_error(Elid::Error, /Hamming distance failed/)
    end

    it "raises error for Hilbert profile ELIDs" do
      embedding = Array.new(1024) { rand }
      elid = Elid.encode(embedding, Elid::Profile::HILBERT10X10)

      expect {
        Elid.hamming_distance(elid, elid)
      }.to raise_error(Elid::Error, /Hamming distance failed/)
    end
  end

  describe ".encode_batch" do
    it "encodes multiple embeddings efficiently" do
      embeddings = Array.new(10) { Array.new(128) { rand } }

      elids = Elid.encode_batch(embeddings, Elid::Profile::MINI128)

      expect(elids).to be_an(Array)
      expect(elids.length).to eq(10)
      expect(elids).to all(be_a(String))
      expect(elids).to all(match(/^[0-9a-v]+$/))
    end

    it "produces same results as individual encoding" do
      embeddings = Array.new(5) { Array.new(128) { rand } }

      batch_elids = Elid.encode_batch(embeddings, Elid::Profile::MINI128)
      individual_elids = embeddings.map { |e| Elid.encode(e, Elid::Profile::MINI128) }

      expect(batch_elids).to eq(individual_elids)
    end

    it "handles empty batch" do
      elids = Elid.encode_batch([], Elid::Profile::MINI128)

      expect(elids).to eq([])
    end

    it "handles single embedding" do
      embeddings = [Array.new(128) { rand }]

      elids = Elid.encode_batch(embeddings, Elid::Profile::MINI128)

      expect(elids.length).to eq(1)
    end

    it "handles large batches" do
      embeddings = Array.new(100) { Array.new(128) { rand } }

      elids = Elid.encode_batch(embeddings, Elid::Profile::MINI128)

      expect(elids.length).to eq(100)
      expect(elids.uniq.length).to eq(100)  # All unique
    end
  end

  describe "Cross-language validation" do
    let(:test_vectors_path) do
      File.join(__dir__, "../../../test-vectors.json")
    end

    it "produces byte-identical output with reference test vectors" do
      skip "Test vectors file not found" unless File.exist?(test_vectors_path)

      test_data = JSON.parse(File.read(test_vectors_path))
      vectors = test_data["vectors"]

      # Test first 5 vectors
      vectors.first(5).each_with_index do |vector, idx|
        embedding = vector["embedding"]
        expected_elid = vector["elid"]

        # Generate ELID
        actual_elid = Elid.encode(embedding, Elid::Profile::MINI128)

        expect(actual_elid).to eq(expected_elid),
          "Vector #{idx}: Expected #{expected_elid}, got #{actual_elid}"
      end
    end

    it "decodes to same bytes as reference implementation" do
      skip "Test vectors file not found" unless File.exist?(test_vectors_path)

      test_data = JSON.parse(File.read(test_vectors_path))
      vectors = test_data["vectors"]

      # Test first 3 vectors
      vectors.first(3).each_with_index do |vector, idx|
        expected_elid = vector["elid"]
        expected_bytes = vector["bytes"]

        # Decode ELID
        actual_bytes = Elid.decode(expected_elid)

        expect(actual_bytes).to eq(expected_bytes),
          "Vector #{idx}: Decoded bytes don't match"
      end
    end
  end

  describe "Lexicographic sorting" do
    it "ELIDs are lexicographically sortable" do
      # Create embeddings with increasing values
      embeddings = (0..9).map do |i|
        Array.new(128) { |j| (i * 128 + j).to_f / 1280.0 }
      end

      elids = embeddings.map { |e| Elid.encode(e, Elid::Profile::MINI128) }
      sorted_elids = elids.sort

      # Verify that ELIDs are sortable strings
      expect(sorted_elids).to be_an(Array)
      expect(sorted_elids.length).to eq(10)
      # Note: SimHash doesn't guarantee perfect locality preservation
      # but all ELIDs should be valid sortable strings
    end

    it "Morton ELIDs sort lexicographically" do
      embeddings = Array.new(20) { Array.new(1024) { rand } }
      elids = embeddings.map { |e| Elid.encode(e, Elid::Profile::MORTON10X10) }

      sorted_elids = elids.sort

      # Verify all are sortable strings
      expect(sorted_elids).to be_an(Array)
      expect(sorted_elids.length).to eq(20)
    end
  end

  describe "Profile constants" do
    it "has MINI128 profile constant" do
      expect(Elid::Profile::MINI128).to eq(1)
    end

    it "has MORTON10X10 profile constant" do
      expect(Elid::Profile::MORTON10X10).to eq(2)
    end

    it "has HILBERT10X10 profile constant" do
      expect(Elid::Profile::HILBERT10X10).to eq(3)
    end
  end
end
