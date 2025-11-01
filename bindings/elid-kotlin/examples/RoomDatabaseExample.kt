package com.elid.examples

import androidx.room.*
import com.elid.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.util.UUID

/**
 * Example demonstrating ELID integration with Android Room Database.
 *
 * This example shows how to:
 * 1. Store ELIDs in a Room database
 * 2. Perform range queries using ELID's lexicographic sortability
 * 3. Find similar embeddings using Hamming distance
 * 4. Use batch operations for efficient encoding
 */

// ========== Entity Definition ==========

@Entity(
    tableName = "embeddings",
    indices = [
        Index(value = ["elid"]),  // Index for range queries
        Index(value = ["profile"]),
        Index(value = ["created_at"])
    ]
)
data class EmbeddingEntity(
    @PrimaryKey
    val id: String = UUID.randomUUID().toString(),

    @ColumnInfo(name = "elid")
    val elid: String,

    @ColumnInfo(name = "profile")
    val profile: String,

    @ColumnInfo(name = "vector_dimension")
    val vectorDimension: Int,

    @ColumnInfo(name = "metadata")
    val metadata: String? = null,

    @ColumnInfo(name = "created_at")
    val createdAt: Long = System.currentTimeMillis()
)

// ========== DAO (Data Access Object) ==========

@Dao
interface EmbeddingDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(entity: EmbeddingEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertAll(entities: List<EmbeddingEntity>)

    @Query("SELECT * FROM embeddings WHERE id = :id")
    suspend fun findById(id: String): EmbeddingEntity?

    @Query("SELECT * FROM embeddings WHERE profile = :profile ORDER BY elid")
    suspend fun findAllByProfile(profile: String): List<EmbeddingEntity>

    /**
     * Range query using lexicographic ELID ordering.
     * This efficiently finds ELIDs in a specific region of the embedding space.
     */
    @Query("""
        SELECT * FROM embeddings
        WHERE elid >= :startElid AND elid < :endElid
        ORDER BY elid
        LIMIT :limit
    """)
    suspend fun findByElidRange(
        startElid: String,
        endElid: String,
        limit: Int = 100
    ): List<EmbeddingEntity>

    /**
     * Find ELIDs with a specific prefix (useful for hierarchical clustering).
     */
    @Query("""
        SELECT * FROM embeddings
        WHERE elid LIKE :prefix || '%'
        ORDER BY elid
        LIMIT :limit
    """)
    suspend fun findByElidPrefix(prefix: String, limit: Int = 100): List<EmbeddingEntity>

    @Query("DELETE FROM embeddings WHERE id = :id")
    suspend fun deleteById(id: String)

    @Query("DELETE FROM embeddings")
    suspend fun deleteAll()

    @Query("SELECT COUNT(*) FROM embeddings")
    suspend fun count(): Int
}

// ========== Database Definition ==========

@Database(
    entities = [EmbeddingEntity::class],
    version = 1,
    exportSchema = false
)
abstract class EmbeddingDatabase : RoomDatabase() {
    abstract fun embeddingDao(): EmbeddingDao

    companion object {
        @Volatile
        private var INSTANCE: EmbeddingDatabase? = null

        fun getInstance(context: android.content.Context): EmbeddingDatabase {
            return INSTANCE ?: synchronized(this) {
                val instance = Room.databaseBuilder(
                    context.applicationContext,
                    EmbeddingDatabase::class.java,
                    "embedding_database"
                )
                    .fallbackToDestructiveMigration()
                    .build()
                INSTANCE = instance
                instance
            }
        }
    }
}

// ========== Repository with ELID Operations ==========

class EmbeddingRepository(private val dao: EmbeddingDao) {

    /**
     * Store a single embedding as an ELID.
     */
    suspend fun storeEmbedding(
        embedding: List<Float>,
        profile: ElidProfile = ElidProfile.MINI_128,
        metadata: String? = null
    ): String = withContext(Dispatchers.IO) {
        val elid = Elid.from(embedding, profile)
        val entity = EmbeddingEntity(
            elid = elid.value,
            profile = profile.name,
            vectorDimension = embedding.size,
            metadata = metadata
        )
        dao.insert(entity)
        entity.id
    }

    /**
     * Store multiple embeddings in batch (efficient parallel encoding).
     */
    suspend fun storeBatch(
        embeddings: List<List<Float>>,
        profile: ElidProfile = ElidProfile.MINI_128,
        metadataList: List<String?>? = null
    ): List<String> = withContext(Dispatchers.IO) {
        // Encode all embeddings in parallel
        val elids = encodeBatch(embeddings, profile)

        // Create entities
        val entities = elids.mapIndexed { index, elidString ->
            EmbeddingEntity(
                elid = elidString,
                profile = profile.name,
                vectorDimension = embeddings[index].size,
                metadata = metadataList?.getOrNull(index)
            )
        }

        dao.insertAll(entities)
        entities.map { it.id }
    }

    /**
     * Find similar embeddings using Hamming distance threshold.
     * This is a two-stage process:
     * 1. Narrow search space using range query (fast, database-indexed)
     * 2. Compute exact Hamming distances (slower, but on smaller set)
     */
    suspend fun findSimilar(
        queryEmbedding: List<Float>,
        threshold: UInt = 20u,
        maxResults: Int = 50
    ): List<Pair<EmbeddingEntity, UInt>> = withContext(Dispatchers.IO) {
        require(threshold <= 128u) { "Threshold must be <= 128 for Mini128" }

        // Encode query
        val queryElid = Elid.from(queryEmbedding, ElidProfile.MINI_128)

        // Stage 1: Narrow search using range query
        // Use prefix matching to get candidates (rough approximation)
        val prefix = queryElid.value.take(4) // Use first 4 chars as prefix
        val candidates = dao.findByElidPrefix(prefix, limit = maxResults * 5)

        // Stage 2: Compute exact Hamming distances
        candidates
            .mapNotNull { entity ->
                try {
                    val entityElid = Elid.parse(entity.elid)
                    val distance = queryElid.hammingDistanceTo(entityElid)
                    if (distance <= threshold) entity to distance else null
                } catch (e: Exception) {
                    null // Skip invalid ELIDs
                }
            }
            .sortedBy { it.second } // Sort by distance
            .take(maxResults)
    }

    /**
     * Find embeddings in a specific ELID range (useful for spatial queries).
     */
    suspend fun findInRange(
        startElid: String,
        endElid: String,
        limit: Int = 100
    ): List<EmbeddingEntity> = withContext(Dispatchers.IO) {
        dao.findByElidRange(startElid, endElid, limit)
    }

    /**
     * Get embedding by ID.
     */
    suspend fun getById(id: String): EmbeddingEntity? = withContext(Dispatchers.IO) {
        dao.findById(id)
    }

    /**
     * Delete embedding by ID.
     */
    suspend fun delete(id: String) = withContext(Dispatchers.IO) {
        dao.deleteById(id)
    }

    /**
     * Get statistics about stored embeddings.
     */
    suspend fun getStats(): DatabaseStats = withContext(Dispatchers.IO) {
        val total = dao.count()
        val byProfile = dao.findAllByProfile("MINI_128").size
        DatabaseStats(
            totalEmbeddings = total,
            mini128Count = byProfile
        )
    }
}

data class DatabaseStats(
    val totalEmbeddings: Int,
    val mini128Count: Int
)

// ========== Usage Examples ==========

class EmbeddingUsageExamples(private val repository: EmbeddingRepository) {

    /**
     * Example 1: Store and retrieve a single embedding.
     */
    suspend fun example1_storeAndRetrieve() {
        // Create a sample 384-dimensional embedding
        val embedding = List(384) { it.toFloat() / 384.0f }

        // Store it
        val id = repository.storeEmbedding(
            embedding = embedding,
            profile = ElidProfile.MINI_128,
            metadata = """{"source": "user_query", "timestamp": ${System.currentTimeMillis()}}"""
        )

        // Retrieve it
        val entity = repository.getById(id)
        println("Stored embedding with ELID: ${entity?.elid}")
    }

    /**
     * Example 2: Batch insert for efficient storage.
     */
    suspend fun example2_batchInsert() {
        val embeddings = List(100) { index ->
            List(384) { (it + index * 10).toFloat() / 384.0f }
        }

        val metadata = embeddings.mapIndexed { index, _ ->
            """{"batch": "import_2024", "index": $index}"""
        }

        val ids = repository.storeBatch(
            embeddings = embeddings,
            profile = ElidProfile.MINI_128,
            metadataList = metadata
        )

        println("Inserted ${ids.size} embeddings in batch")
    }

    /**
     * Example 3: Similarity search using Hamming distance.
     */
    suspend fun example3_similaritySearch() {
        val queryEmbedding = List(384) { it.toFloat() / 384.0f }

        // Find embeddings within Hamming distance of 15
        val results = repository.findSimilar(
            queryEmbedding = queryEmbedding,
            threshold = 15u,
            maxResults = 10
        )

        results.forEach { (entity, distance) ->
            println("Found similar embedding (distance=$distance): ${entity.elid}")
        }
    }

    /**
     * Example 4: Range query using ELID's lexicographic ordering.
     */
    suspend fun example4_rangeQuery() {
        // Find all ELIDs between "0a" and "0b" (these share a prefix)
        val results = repository.findInRange(
            startElid = "0a",
            endElid = "0b",
            limit = 50
        )

        println("Found ${results.size} embeddings in range")
    }

    /**
     * Example 5: Deduplication using ELID comparison.
     */
    suspend fun example5_deduplication() {
        val newEmbedding = List(384) { it.toFloat() / 384.0f }
        val newElid = Elid.from(newEmbedding, ElidProfile.MINI_128)

        // Check if a very similar embedding already exists
        val similar = repository.findSimilar(
            queryEmbedding = newEmbedding,
            threshold = 5u, // Very strict threshold
            maxResults = 1
        )

        if (similar.isNotEmpty()) {
            println("Duplicate detected! Existing ELID: ${similar[0].first.elid}")
        } else {
            // Store new embedding
            repository.storeEmbedding(newEmbedding, metadata = """{"status": "unique"}""")
            println("New unique embedding stored")
        }
    }

    /**
     * Example 6: Hierarchical clustering using ELID prefixes.
     */
    suspend fun example6_clustering() {
        // Get all embeddings
        val allEmbeddings = repository.findInRange("0", "g", limit = 1000)

        // Group by ELID prefix (first 3 characters)
        val clusters = allEmbeddings.groupBy { it.elid.take(3) }

        clusters.forEach { (prefix, members) ->
            println("Cluster $prefix: ${members.size} embeddings")
        }
    }
}

// ========== ViewModel Integration (Android) ==========

/*
class EmbeddingViewModel(
    private val repository: EmbeddingRepository
) : ViewModel() {

    private val _searchResults = MutableLiveData<List<Pair<EmbeddingEntity, UInt>>>()
    val searchResults: LiveData<List<Pair<EmbeddingEntity, UInt>>> = _searchResults

    private val _isLoading = MutableLiveData<Boolean>()
    val isLoading: LiveData<Boolean> = _isLoading

    fun searchSimilar(embedding: List<Float>, threshold: UInt = 20u) {
        viewModelScope.launch {
            _isLoading.value = true
            try {
                val results = repository.findSimilar(embedding, threshold)
                _searchResults.value = results
            } catch (e: Exception) {
                // Handle error
                Log.e("EmbeddingViewModel", "Search failed", e)
            } finally {
                _isLoading.value = false
            }
        }
    }

    fun storeEmbedding(embedding: List<Float>, metadata: String?) {
        viewModelScope.launch {
            try {
                repository.storeEmbedding(embedding, metadata = metadata)
            } catch (e: Exception) {
                Log.e("EmbeddingViewModel", "Store failed", e)
            }
        }
    }
}
*/
