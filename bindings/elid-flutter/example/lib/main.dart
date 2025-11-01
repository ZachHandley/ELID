import 'package:flutter/material.dart';
import 'package:elid/elid.dart';
import 'dart:async';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await Elid.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'ELID Flutter Demo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const MyHomePage(title: 'ELID Flutter Demo'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  String _encodedElid = '';
  double _batchProgress = 0.0;
  int _streamCount = 0;
  bool _isProcessing = false;
  String _statusMessage = 'Ready';
  final List<String> _log = [];

  void _addLog(String message) {
    setState(() {
      _log.add('${DateTime.now().toIso8601String()}: $message');
      if (_log.length > 10) {
        _log.removeAt(0);
      }
    });
  }

  Future<void> _encodeSingle() async {
    setState(() {
      _isProcessing = true;
      _statusMessage = 'Encoding single embedding...';
    });

    try {
      final embedding = List.generate(768, (i) => i * 0.001);
      final stopwatch = Stopwatch()..start();
      final elid = Elid.encode(embedding, Profile.mini128);
      stopwatch.stop();

      setState(() {
        _encodedElid = elid;
        _statusMessage = 'Encoded in ${stopwatch.elapsedMicroseconds}μs';
      });

      _addLog('Single encode: ${stopwatch.elapsedMicroseconds}μs');
    } on ElidException catch (e) {
      setState(() {
        _statusMessage = 'Error: ${e.message}';
      });
      _addLog('Error: ${e.message}');
    } finally {
      setState(() {
        _isProcessing = false;
      });
    }
  }

  Future<void> _encodeBatch() async {
    setState(() {
      _isProcessing = true;
      _batchProgress = 0.0;
      _statusMessage = 'Encoding batch...';
    });

    try {
      // Generate 5000 embeddings
      final embeddings = List.generate(
        5000,
        (i) => List.generate(768, (j) => (i + j) * 0.0001),
      );

      final stopwatch = Stopwatch()..start();
      final elids = await Elid.encodeBatch(embeddings, Profile.mini128);
      stopwatch.stop();

      setState(() {
        _batchProgress = 1.0;
        _statusMessage =
            'Batch complete: ${elids.length} ELIDs in ${stopwatch.elapsedMilliseconds}ms';
      });

      _addLog(
          'Batch encode: ${elids.length} in ${stopwatch.elapsedMilliseconds}ms');
    } on ElidException catch (e) {
      setState(() {
        _statusMessage = 'Error: ${e.message}';
      });
      _addLog('Error: ${e.message}');
    } finally {
      setState(() {
        _isProcessing = false;
      });
    }
  }

  Future<void> _encodeStream() async {
    setState(() {
      _isProcessing = true;
      _streamCount = 0;
      _statusMessage = 'Streaming encoding...';
    });

    try {
      // Generate 1000 embeddings
      final embeddings = List.generate(
        1000,
        (i) => List.generate(768, (j) => (i + j) * 0.0001),
      );

      final stopwatch = Stopwatch()..start();
      await for (final elid
          in Elid.encodeStream(embeddings, Profile.mini128)) {
        setState(() {
          _streamCount++;
          _statusMessage = 'Encoded $_streamCount/${embeddings.length}';
        });

        // Small delay to show progress visually
        if (_streamCount % 10 == 0) {
          await Future.delayed(const Duration(milliseconds: 1));
        }
      }
      stopwatch.stop();

      setState(() {
        _statusMessage =
            'Stream complete: $_streamCount ELIDs in ${stopwatch.elapsedMilliseconds}ms';
      });

      _addLog(
          'Stream encode: $_streamCount in ${stopwatch.elapsedMilliseconds}ms');
    } on ElidException catch (e) {
      setState(() {
        _statusMessage = 'Error: ${e.message}';
      });
      _addLog('Error: ${e.message}');
    } finally {
      setState(() {
        _isProcessing = false;
      });
    }
  }

  Future<void> _testSimilarity() async {
    setState(() {
      _isProcessing = true;
      _statusMessage = 'Testing similarity...';
    });

    try {
      // Create two similar embeddings
      final embedding1 = List.generate(768, (i) => i * 0.001);
      final embedding2 = List.generate(768, (i) => i * 0.001 + 0.01);

      final elid1 = Elid.encode(embedding1, Profile.mini128);
      final elid2 = Elid.encode(embedding2, Profile.mini128);

      final distance = Elid.hammingDistance(elid1, elid2);

      setState(() {
        _statusMessage = 'Hamming distance: $distance/128';
      });

      _addLog('Similarity test: distance=$distance');
    } on ElidException catch (e) {
      setState(() {
        _statusMessage = 'Error: ${e.message}';
      });
      _addLog('Error: ${e.message}');
    } finally {
      setState(() {
        _isProcessing = false;
      });
    }
  }

  Future<void> _testAllProfiles() async {
    setState(() {
      _isProcessing = true;
      _statusMessage = 'Testing all profiles...';
    });

    try {
      final embedding = List.generate(768, (i) => i * 0.001);

      final mini128 = Elid.encode(embedding, Profile.mini128);
      final morton = Elid.encode(embedding, Profile.morton10x10);
      final hilbert = Elid.encode(embedding, Profile.hilbert10x10);

      setState(() {
        _statusMessage = 'All profiles tested successfully';
      });

      _addLog('Mini128: ${mini128.length} chars');
      _addLog('Morton10x10: ${morton.length} chars');
      _addLog('Hilbert10x10: ${hilbert.length} chars');
    } on ElidException catch (e) {
      setState(() {
        _statusMessage = 'Error: ${e.message}';
      });
      _addLog('Error: ${e.message}');
    } finally {
      setState(() {
        _isProcessing = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Status',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(_statusMessage),
                    if (_isProcessing) ...[
                      const SizedBox(height: 8),
                      const LinearProgressIndicator(),
                    ],
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            if (_encodedElid.isNotEmpty)
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Encoded ELID',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      const SizedBox(height: 8),
                      Text(
                        _encodedElid,
                        style: const TextStyle(fontFamily: 'monospace'),
                      ),
                    ],
                  ),
                ),
              ),
            const SizedBox(height: 16),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                ElevatedButton(
                  onPressed: _isProcessing ? null : _encodeSingle,
                  child: const Text('Encode Single'),
                ),
                ElevatedButton(
                  onPressed: _isProcessing ? null : _encodeBatch,
                  child: const Text('Batch (5000)'),
                ),
                ElevatedButton(
                  onPressed: _isProcessing ? null : _encodeStream,
                  child: const Text('Stream (1000)'),
                ),
                ElevatedButton(
                  onPressed: _isProcessing ? null : _testSimilarity,
                  child: const Text('Test Similarity'),
                ),
                ElevatedButton(
                  onPressed: _isProcessing ? null : _testAllProfiles,
                  child: const Text('All Profiles'),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Expanded(
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Log',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      const SizedBox(height: 8),
                      Expanded(
                        child: ListView.builder(
                          itemCount: _log.length,
                          itemBuilder: (context, index) {
                            return Text(
                              _log[index],
                              style: const TextStyle(
                                fontFamily: 'monospace',
                                fontSize: 12,
                              ),
                            );
                          },
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
