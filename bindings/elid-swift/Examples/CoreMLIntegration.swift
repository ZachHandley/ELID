import CoreML
import Vision
import Elid
import Foundation

#if canImport(UIKit)
import UIKit
typealias PlatformImage = UIImage
#elseif canImport(AppKit)
import AppKit
typealias PlatformImage = NSImage
#endif

// MARK: - Example 1: Using Vision Framework for Image Embeddings

/// Generate embeddings using Apple's Vision framework
class VisionEmbeddingGenerator {

    func generateEmbedding(for image: PlatformImage) async throws -> [Float] {
        #if canImport(UIKit)
        guard let cgImage = image.cgImage else {
            throw EmbeddingError.invalidImage
        }
        #elseif canImport(AppKit)
        guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
            throw EmbeddingError.invalidImage
        }
        #endif

        // Create Vision request for feature extraction
        let request = VNGenerateImageFeaturePrintRequest()
        let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])

        // Perform request
        try handler.perform([request])

        guard let observation = request.results?.first else {
            throw EmbeddingError.noFeatures
        }

        // Convert VNFeaturePrintObservation to Float array
        let featureData = observation.data
        let count = featureData.count / MemoryLayout<Float>.stride

        return featureData.withUnsafeBytes { buffer in
            Array(UnsafeBufferPointer<Float>(
                start: buffer.baseAddress!.assumingMemoryBound(to: Float.self),
                count: count
            ))
        }
    }

    func generateElid(for image: PlatformImage, profile: UniProfile = .mini128) async throws -> String {
        let embedding = try await generateEmbedding(for: image)
        return try uniEncode(embedding: embedding, profile: profile)
    }
}

// MARK: - Example 2: Custom CoreML Model Integration

/// Wrapper for a custom CoreML embedding model
class CoreMLEmbeddingModel {
    private let model: MLModel

    init(modelURL: URL) throws {
        let config = MLModelConfiguration()
        #if !targetEnvironment(simulator)
        // Use Neural Engine on device
        config.computeUnits = .all
        #else
        // CPU only on simulator
        config.computeUnits = .cpuOnly
        #endif

        self.model = try MLModel(contentsOf: modelURL, configuration: config)
    }

    /// Generate embedding from text input
    /// Assumes model has input "text" and output "embedding"
    func generateTextEmbedding(text: String) throws -> [Float] {
        let input = try MLDictionaryFeatureProvider(dictionary: [
            "text": text
        ])

        let output = try model.prediction(from: input)

        // Extract embedding from model output
        guard let embeddingValue = output.featureValue(for: "embedding"),
              let multiArray = embeddingValue.multiArrayValue else {
            throw EmbeddingError.invalidModelOutput
        }

        return multiArray.toFloatArray()
    }

    /// Generate ELID from text
    func generateElid(text: String, profile: UniProfile = .mini128) throws -> String {
        let embedding = try generateTextEmbedding(text: text)
        return try uniEncode(embedding: embedding, profile: profile)
    }
}

// MARK: - Example 3: Semantic Search with SwiftData

#if canImport(SwiftData)
import SwiftData

@Model
class Document {
    @Attribute(.unique) var id: String // ELID
    var content: String
    var embedding: Data // Store raw embedding for re-computation
    var createdAt: Date

    init(content: String, embedding: [Float]) throws {
        self.id = try uniEncode(embedding: embedding, profile: .mini128)
        self.content = content
        self.embedding = Data(embedding.flatMap { withUnsafeBytes(of: $0, Array.init) })
        self.createdAt = Date()
    }

    /// Compute Hamming distance to another document
    func distance(to other: Document) throws -> UInt32 {
        try uniHammingDistance(elid1: self.id, elid2: other.id)
    }
}

class SemanticSearchService {
    private let modelContainer: ModelContainer
    private let embeddingModel: CoreMLEmbeddingModel

    init(modelContainer: ModelContainer, embeddingModelURL: URL) throws {
        self.modelContainer = modelContainer
        self.embeddingModel = try CoreMLEmbeddingModel(modelURL: embeddingModelURL)
    }

    /// Add a document to the search index
    @MainActor
    func indexDocument(content: String) throws {
        let context = modelContainer.mainContext
        let embedding = try embeddingModel.generateTextEmbedding(text: content)
        let document = try Document(content: content, embedding: embedding)
        context.insert(document)
        try context.save()
    }

    /// Search for similar documents
    @MainActor
    func search(query: String, maxDistance: UInt32 = 25, limit: Int = 10) throws -> [Document] {
        let context = modelContainer.mainContext

        // Generate ELID for query
        let queryElid = try embeddingModel.generateElid(text: query)

        // Fetch all documents (in production, use prefix-based filtering)
        let descriptor = FetchDescriptor<Document>(sortBy: [SortDescriptor(\.createdAt, order: .reverse)])
        let allDocuments = try context.fetch(descriptor)

        // Compute distances and filter
        let results = try allDocuments
            .map { doc in
                (doc, try uniHammingDistance(elid1: queryElid, elid2: doc.id))
            }
            .filter { $0.1 <= maxDistance }
            .sorted { $0.1 < $1.1 }
            .prefix(limit)
            .map { $0.0 }

        return Array(results)
    }

    /// Find similar documents using prefix-based optimization
    /// ELIDs with common prefixes are more similar
    @MainActor
    func searchWithPrefix(query: String, prefixLength: Int = 4, limit: Int = 10) throws -> [Document] {
        let context = modelContainer.mainContext

        // Generate ELID for query
        let queryElid = try embeddingModel.generateElid(text: query)
        let prefix = String(queryElid.prefix(prefixLength))

        // Fetch only documents with matching prefix (exploits sortability)
        let descriptor = FetchDescriptor<Document>(
            predicate: #Predicate { $0.id.hasPrefix(prefix) },
            sortBy: [SortDescriptor(\.id)]
        )
        let candidates = try context.fetch(descriptor)

        // Refine with Hamming distance
        let results = try candidates
            .map { doc in
                (doc, try uniHammingDistance(elid1: queryElid, elid2: doc.id))
            }
            .sorted { $0.1 < $1.1 }
            .prefix(limit)
            .map { $0.0 }

        return Array(results)
    }
}
#endif

// MARK: - Example 4: Image Similarity Clustering

class ImageSimilarityClusterer {
    private let generator: VisionEmbeddingGenerator

    init() {
        self.generator = VisionEmbeddingGenerator()
    }

    /// Cluster images by similarity using Hamming distance
    func cluster(images: [PlatformImage], threshold: UInt32 = 20) async throws -> [[PlatformImage]] {
        // Generate ELIDs for all images
        let imageElids = try await withThrowingTaskGroup(of: (Int, String).self) { group in
            for (index, image) in images.enumerated() {
                group.addTask {
                    let elid = try await self.generator.generateElid(for: image)
                    return (index, elid)
                }
            }

            var results = [(Int, String)]()
            for try await result in group {
                results.append(result)
            }
            return results.sorted { $0.0 < $1.0 }.map { $0.1 }
        }

        // Cluster using Hamming distance
        var clusters: [[Int]] = []
        var assigned = Set<Int>()

        for (i, elid1) in imageElids.enumerated() {
            guard !assigned.contains(i) else { continue }

            var cluster = [i]
            assigned.insert(i)

            for (j, elid2) in imageElids.enumerated() where j > i {
                guard !assigned.contains(j) else { continue }

                let distance = try uniHammingDistance(elid1: elid1, elid2: elid2)
                if distance <= threshold {
                    cluster.append(j)
                    assigned.insert(j)
                }
            }

            clusters.append(cluster)
        }

        // Convert index clusters to image clusters
        return clusters.map { cluster in
            cluster.map { images[$0] }
        }
    }
}

// MARK: - Example 5: Real-time Duplicate Detection

class DuplicateDetector {
    private var knownImages: [String: PlatformImage] = [:] // ELID -> Image
    private let threshold: UInt32
    private let generator: VisionEmbeddingGenerator

    init(threshold: UInt32 = 15) {
        self.threshold = threshold
        self.generator = VisionEmbeddingGenerator()
    }

    /// Check if image is a duplicate of any known image
    func isDuplicate(image: PlatformImage) async throws -> (Bool, PlatformImage?, UInt32?) {
        let elid = try await generator.generateElid(for: image)

        // Check against all known images
        for (knownElid, knownImage) in knownImages {
            let distance = try uniHammingDistance(elid1: elid, elid2: knownElid)
            if distance <= threshold {
                return (true, knownImage, distance)
            }
        }

        // Not a duplicate, add to known images
        knownImages[elid] = image
        return (false, nil, nil)
    }

    /// Add image without duplicate check
    func addImage(_ image: PlatformImage) async throws {
        let elid = try await generator.generateElid(for: image)
        knownImages[elid] = image
    }
}

// MARK: - Example 6: Batch Processing Pipeline

class BatchEmbeddingPipeline {

    /// Process a batch of embeddings and store with metadata
    struct EmbeddingRecord {
        let elid: String
        let metadata: [String: String]
        let timestamp: Date
    }

    func processBatch(embeddings: [[Float]], metadata: [[String: String]]) throws -> [EmbeddingRecord] {
        guard embeddings.count == metadata.count else {
            throw EmbeddingError.batchSizeMismatch
        }

        // Use batch encoding for better performance
        let elids = try uniEncodeBatch(embeddings: embeddings, profile: .mini128)

        return zip(elids, metadata).map { elid, meta in
            EmbeddingRecord(elid: elid, metadata: meta, timestamp: Date())
        }
    }

    /// Process images in parallel batches
    func processImages(_ images: [PlatformImage], batchSize: Int = 10) async throws -> [EmbeddingRecord] {
        let generator = VisionEmbeddingGenerator()
        var allRecords: [EmbeddingRecord] = []

        // Process in batches
        for batchStart in stride(from: 0, to: images.count, by: batchSize) {
            let batchEnd = min(batchStart + batchSize, images.count)
            let batch = Array(images[batchStart..<batchEnd])

            // Generate embeddings in parallel
            let embeddings = try await withThrowingTaskGroup(of: (Int, [Float]).self) { group in
                for (index, image) in batch.enumerated() {
                    group.addTask {
                        let embedding = try await generator.generateEmbedding(for: image)
                        return (index, embedding)
                    }
                }

                var results = [(Int, [Float])]()
                for try await result in group {
                    results.append(result)
                }
                return results.sorted { $0.0 < $1.0 }.map { $0.1 }
            }

            // Batch encode
            let metadata = batch.enumerated().map { index, _ in
                ["batch": String(batchStart / batchSize), "index": String(index)]
            }
            let records = try processBatch(embeddings: embeddings, metadata: metadata)
            allRecords.append(contentsOf: records)
        }

        return allRecords
    }
}

// MARK: - Supporting Types

enum EmbeddingError: Error, LocalizedError {
    case invalidImage
    case noFeatures
    case invalidModelOutput
    case batchSizeMismatch

    var errorDescription: String? {
        switch self {
        case .invalidImage:
            return "Could not extract CGImage from image"
        case .noFeatures:
            return "Vision framework could not extract features"
        case .invalidModelOutput:
            return "CoreML model output does not contain expected embedding"
        case .batchSizeMismatch:
            return "Number of embeddings does not match number of metadata entries"
        }
    }
}

// MARK: - Helper Extensions

extension MLMultiArray {
    func toFloatArray() -> [Float] {
        let count = self.count
        var result = [Float](repeating: 0, count: count)

        for i in 0..<count {
            result[i] = Float(truncating: self[i])
        }

        return result
    }
}

// MARK: - Usage Example

/*

 // Example 1: Vision Framework
 let generator = VisionEmbeddingGenerator()
 let image = UIImage(named: "photo.jpg")!
 let elid = try await generator.generateElid(for: image)
 print("Image ELID: \(elid)")

 // Example 2: Custom CoreML Model
 let modelURL = Bundle.main.url(forResource: "TextEmbedding", withExtension: "mlmodelc")!
 let model = try CoreMLEmbeddingModel(modelURL: modelURL)
 let textElid = try model.generateElid(text: "Hello, world!")

 // Example 3: Semantic Search
 let searchService = try SemanticSearchService(
     modelContainer: modelContainer,
     embeddingModelURL: modelURL
 )
 try await searchService.indexDocument(content: "Machine learning is fascinating")
 let results = try await searchService.search(query: "AI and ML", maxDistance: 20)

 // Example 4: Image Clustering
 let clusterer = ImageSimilarityClusterer()
 let clusters = try await clusterer.cluster(images: photos, threshold: 25)
 print("Found \(clusters.count) clusters")

 // Example 5: Duplicate Detection
 let detector = DuplicateDetector(threshold: 15)
 let (isDupe, original, distance) = try await detector.isDuplicate(image: newImage)
 if isDupe {
     print("Duplicate found! Distance: \(distance!)")
 }

 // Example 6: Batch Processing
 let pipeline = BatchEmbeddingPipeline()
 let records = try await pipeline.processImages(photos, batchSize: 20)
 print("Processed \(records.count) images")

 */
