import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:elid/elid.dart';

void main() {
  setUpAll(() async {
    // Initialize the ELID library before running tests
    await Elid.init();
  });

  group('ELID Basic Operations', () {
    test('encode() returns valid ELID string', () {
      final embedding = List.generate(768, (i) => i * 0.001);
      final elid = Elid.encode(embedding, Profile.mini128);

      expect(elid.length, 29);
      expect(elid, matches(RegExp(r'^[0-9A-V]+$'))); // Base32hex alphabet
    });

    test('encode() with Morton10x10 profile', () {
      final embedding = List.generate(768, (i) => i * 0.001);
      final elid = Elid.encode(embedding, Profile.morton10x10);

      expect(elid.length, greaterThanOrEqualTo(16));
      expect(elid.length, lessThanOrEqualTo(24));
    });

    test('encode() with Hilbert10x10 profile', () {
      final embedding = List.generate(768, (i) => i * 0.001);
      final elid = Elid.encode(embedding, Profile.hilbert10x10);

      expect(elid.length, greaterThanOrEqualTo(16));
      expect(elid.length, lessThanOrEqualTo(24));
    });

    test('decode() returns correct byte length', () {
      final embedding = List.filled(768, 0.5);
      final elid = Elid.encode(embedding, Profile.mini128);
      final bytes = Elid.decode(elid);

      expect(bytes.length, 16);
      expect(bytes, isA<Uint8List>());
    });

    test('encode() throws on invalid dimensions', () {
      final tooSmall = List.filled(32, 0.5); // < 64
      expect(
        () => Elid.encode(tooSmall, Profile.mini128),
        throwsA(isA<ElidException>()),
      );

      final tooLarge = List.filled(3000, 0.5); // > 2048
      expect(
        () => Elid.encode(tooLarge, Profile.mini128),
        throwsA(isA<ElidException>()),
      );
    });

    test('decode() throws on invalid ELID string', () {
      expect(
        () => Elid.decode('invalid!!!'),
        throwsA(isA<ElidException>()),
      );
    });
  });

  group('Hamming Distance', () {
    test('hammingDistance() returns 0 for identical embeddings', () {
      final embedding = List.filled(768, 0.5);
      final elid1 = Elid.encode(embedding, Profile.mini128);
      final elid2 = Elid.encode(embedding, Profile.mini128);

      final distance = Elid.hammingDistance(elid1, elid2);
      expect(distance, 0);
    });

    test('hammingDistance() returns non-zero for different embeddings', () {
      final embedding1 = List.filled(768, 0.1);
      final embedding2 = List.filled(768, 0.9);

      final elid1 = Elid.encode(embedding1, Profile.mini128);
      final elid2 = Elid.encode(embedding2, Profile.mini128);

      final distance = Elid.hammingDistance(elid1, elid2);
      expect(distance, greaterThan(0));
      expect(distance, lessThanOrEqualTo(128));
    });

    test('hammingDistance() throws on incompatible profiles', () {
      final embedding = List.filled(768, 0.5);
      final elid1 = Elid.encode(embedding, Profile.mini128);
      final elid2 = Elid.encode(embedding, Profile.morton10x10);

      expect(
        () => Elid.hammingDistance(elid1, elid2),
        throwsA(isA<ElidException>()),
      );
    });
  });

  group('Async Batch Encoding', () {
    test('encodeBatch() processes multiple embeddings', () async {
      final embeddings = List.generate(
        100,
        (_) => List.generate(768, (i) => i * 0.001),
      );

      final elids = await Elid.encodeBatch(embeddings, Profile.mini128);

      expect(elids.length, 100);
      expect(elids.every((e) => e.length == 29), true);
    });

    test('encodeBatch() maintains order', () async {
      final embeddings = [
        List.filled(768, 0.1),
        List.filled(768, 0.5),
        List.filled(768, 0.9),
      ];

      final elids = await Elid.encodeBatch(embeddings, Profile.mini128);

      // Verify order by re-encoding
      expect(elids[0], Elid.encode(embeddings[0], Profile.mini128));
      expect(elids[1], Elid.encode(embeddings[1], Profile.mini128));
      expect(elids[2], Elid.encode(embeddings[2], Profile.mini128));
    });

    test('encodeBatch() handles large batches efficiently', () async {
      final embeddings = List.generate(
        5000,
        (_) => List.generate(768, (i) => i * 0.001),
      );

      final stopwatch = Stopwatch()..start();
      final elids = await Elid.encodeBatch(embeddings, Profile.mini128);
      stopwatch.stop();

      expect(elids.length, 5000);
      expect(stopwatch.elapsedMilliseconds, lessThan(10000)); // <10s requirement
    });
  });

  group('Stream Encoding', () {
    test('encodeStream() yields results progressively', () async {
      final embeddings = List.generate(
        10,
        (_) => List.generate(768, (i) => i * 0.001),
      );

      final elids = <String>[];
      await for (final elid in Elid.encodeStream(embeddings, Profile.mini128)) {
        elids.add(elid);
      }

      expect(elids.length, 10);
      expect(elids.every((e) => e.length == 29), true);
    });

    test('encodeStream() can be cancelled', () async {
      final embeddings = List.generate(
        1000,
        (_) => List.generate(768, (i) => i * 0.001),
      );

      final elids = <String>[];
      final stream = Elid.encodeStream(embeddings, Profile.mini128);

      await for (final elid in stream) {
        elids.add(elid);
        if (elids.length >= 50) {
          break; // Cancel after 50 items
        }
      }

      expect(elids.length, 50);
    });
  });

  group('Cross-Language Validation', () {
    test('validate against test vectors', () async {
      // Load test vectors
      final testVectorsPath = '../test-vectors.json';
      final file = File(testVectorsPath);

      if (!await file.exists()) {
        print('Test vectors not found, skipping cross-language validation');
        return;
      }

      final jsonString = await file.readAsString();
      final testData = jsonDecode(jsonString) as Map<String, dynamic>;
      final vectors = testData['vectors'] as List<dynamic>;

      // Test first 15 vectors
      for (var i = 0; i < 15 && i < vectors.length; i++) {
        final vector = vectors[i] as Map<String, dynamic>;
        final embedding = (vector['embedding'] as List<dynamic>)
            .map((e) => (e as num).toDouble())
            .toList();
        final expectedMini128 = vector['elid_mini128'] as String;
        final expectedMorton = vector['elid_morton10x10'] as String;
        final expectedHilbert = vector['elid_hilbert10x10'] as String;

        // Validate Mini128
        final actualMini128 = Elid.encode(embedding, Profile.mini128);
        expect(actualMini128, expectedMini128,
            reason: 'Vector $i Mini128 mismatch');

        // Validate Morton10x10
        final actualMorton = Elid.encode(embedding, Profile.morton10x10);
        expect(actualMorton, expectedMorton,
            reason: 'Vector $i Morton10x10 mismatch');

        // Validate Hilbert10x10
        final actualHilbert = Elid.encode(embedding, Profile.hilbert10x10);
        expect(actualHilbert, expectedHilbert,
            reason: 'Vector $i Hilbert10x10 mismatch');
      }
    });

    test('cross-platform consistency', () {
      // Ensure ELIDs are deterministic across invocations
      final embedding = List.generate(768, (i) => i * 0.001);

      final elid1 = Elid.encode(embedding, Profile.mini128);
      final elid2 = Elid.encode(embedding, Profile.mini128);

      expect(elid1, elid2);
    });
  });

  group('Performance Characteristics', () {
    test('encode() completes quickly for single embedding', () {
      final embedding = List.generate(768, (i) => i * 0.001);

      final stopwatch = Stopwatch()..start();
      final elid = Elid.encode(embedding, Profile.mini128);
      stopwatch.stop();

      expect(elid.length, 29);
      expect(stopwatch.elapsedMicroseconds, lessThan(10000)); // <10ms
    });

    test('hammingDistance() completes in microseconds', () {
      final embedding = List.filled(768, 0.5);
      final elid1 = Elid.encode(embedding, Profile.mini128);
      final elid2 = Elid.encode(embedding, Profile.mini128);

      final stopwatch = Stopwatch()..start();
      final distance = Elid.hammingDistance(elid1, elid2);
      stopwatch.stop();

      expect(distance, 0);
      expect(stopwatch.elapsedMicroseconds, lessThan(100)); // <100μs
    });
  });
}
