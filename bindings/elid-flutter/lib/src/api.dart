import 'dart:typed_data';
import 'bridge_generated.dart/frb_generated.dart';

/// Main ELID class providing encoding/decoding functions
class Elid {
  static final _api = RustLib.instance.api;

  /// Initialize the ELID library.
  /// Must be called once before using any other functions.
  ///
  /// Example:
  /// ```dart
  /// await Elid.init();
  /// ```
  static Future<void> init() async {
    await RustLib.init();
    await _api.crateApiInit();
  }

  /// Encode a single embedding to ELID (synchronous).
  ///
  /// - [embedding]: List of doubles with length 64-2048
  /// - [profile]: Encoding profile (mini128, morton10x10, or hilbert10x10)
  /// - Returns: Sortable string identifier (29 chars for Mini128, 16-24 for others)
  /// - Throws: [ElidException] if embedding dimensions are invalid
  ///
  /// Example:
  /// ```dart
  /// final embedding = List.generate(768, (i) => 0.1 * i);
  /// final elidId = Elid.encode(embedding, Profile.mini128);
  /// print(elidId); // 29-character string
  /// ```
  static String encode(List<double> embedding, Profile profile) {
    try {
      return _api.crateApiEncode(embedding: embedding, profile: profile);
    } catch (e) {
      throw ElidException(e.toString());
    }
  }

  /// Decode ELID string to raw bytes (synchronous).
  ///
  /// - [elid]: ELID string identifier (base32hex encoded)
  /// - Returns: List of bytes including 2-byte header (18 bytes for Mini128; 15 for Morton/Hilbert10x10)
  /// - Throws: [ElidException] if ELID string is malformed
  ///
  /// Example:
  /// ```dart
  /// final bytes = Elid.decode('0123456789ABCDEFGHIJKLMNOPQ');
  /// print(bytes.length); // 18 (2 header + 16 payload)
  /// ```
  static Uint8List decode(String elid) {
    try {
      final bytes = _api.crateApiDecode(elid: elid);
      return Uint8List.fromList(bytes);
    } catch (e) {
      throw ElidException(e.toString());
    }
  }

  /// Calculate Hamming distance between two ELIDs (synchronous).
  ///
  /// - [elid1]: First ELID string
  /// - [elid2]: Second ELID string (must use same profile as elid1)
  /// - Returns: Hamming distance (0-128 for Mini128, 0-80 for Morton/Hilbert10x10)
  ///          Lower distance indicates higher similarity.
  /// - Throws: [ElidException] if ELIDs use different profiles or are malformed
  ///
  /// Example:
  /// ```dart
  /// final embedding = List.filled(768, 0.5);
  /// final elid1 = Elid.encode(embedding, Profile.mini128);
  /// final elid2 = Elid.encode(embedding, Profile.mini128);
  /// final distance = Elid.hammingDistance(elid1, elid2);
  /// print(distance); // 0 (identical embeddings)
  /// ```
  static int hammingDistance(String elid1, String elid2) {
    try {
      return _api.crateApiHammingDistance(elid1: elid1, elid2: elid2);
    } catch (e) {
      throw ElidException(e.toString());
    }
  }

  /// Encode multiple embeddings asynchronously (does not block UI thread).
  ///
  /// - [embeddings]: List of embedding vectors, each length 64-2048
  /// - [profile]: Encoding profile for all embeddings
  /// - Returns: Future resolving to list of ELID strings, same order as input
  /// - Throws: [ElidException] if any embedding has invalid dimensions
  ///
  /// Example:
  /// ```dart
  /// final embeddings = List.generate(5000, (_) => List.filled(768, 0.5));
  /// final elids = await Elid.encodeBatch(embeddings, Profile.mini128);
  /// print(elids.length); // 5000
  /// ```
  static Future<List<String>> encodeBatch(
    List<List<double>> embeddings,
    Profile profile,
  ) async {
    try {
      // Convert List<List<double>> to List<Float64List>
      final float64Embeddings = embeddings.map((e) => Float64List.fromList(e)).toList();
      return await _api.crateApiEncodeBatch(embeddings: float64Embeddings, profile: profile);
    } catch (e) {
      throw ElidException(e.toString());
    }
  }

  // TODO: Stream API not yet supported by flutter_rust_bridge code generation
  // Will be added in future update
  //
  // /// Stream-based encoding for real-time UI updates.
  // /// Yields ELID strings progressively as embeddings are encoded.
  // static Stream<String> encodeStream(
  //   List<List<double>> embeddings,
  //   Profile profile,
  // ) { ... }
}

/// Exception thrown by ELID functions
class ElidException implements Exception {
  /// Human-readable error message
  final String message;

  ElidException(this.message);

  @override
  String toString() => 'ElidException: $message';
}
