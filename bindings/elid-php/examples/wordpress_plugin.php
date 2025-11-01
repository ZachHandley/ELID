<?php
/**
 * Plugin Name: ELID Vector Search
 * Plugin URI: https://github.com/yourusername/elid
 * Description: Add locality-preserving identifiers to WordPress posts for vector search
 * Version: 1.0.0
 * Author: Your Name
 * Author URI: https://yourwebsite.com
 * License: MIT
 */

// Prevent direct access
if (!defined('ABSPATH')) {
    exit;
}

// Load Composer autoloader
require_once __DIR__ . '/vendor/autoload.php';

use Elid\Elid;
use Elid\Exception\ElidException;

class ELID_Vector_Search
{
    /**
     * Initialize the plugin
     */
    public static function init()
    {
        // Add custom field to posts
        add_action('add_meta_boxes', [self::class, 'add_meta_box']);
        add_action('save_post', [self::class, 'save_post_meta']);

        // Add REST API endpoint
        add_action('rest_api_init', [self::class, 'register_rest_routes']);

        // Add admin settings page
        add_action('admin_menu', [self::class, 'add_admin_menu']);
    }

    /**
     * Add meta box to post editor
     */
    public static function add_meta_box()
    {
        add_meta_box(
            'elid_vector_meta',
            'ELID Vector Identifier',
            [self::class, 'render_meta_box'],
            'post',
            'side',
            'default'
        );
    }

    /**
     * Render meta box content
     */
    public static function render_meta_box($post)
    {
        $elid = get_post_meta($post->ID, '_elid', true);
        $embedding_available = get_post_meta($post->ID, '_elid_embedding_set', true);

        wp_nonce_field('elid_meta_box', 'elid_meta_box_nonce');

        echo '<div class="elid-meta-box">';

        if ($elid) {
            echo '<p><strong>ELID:</strong></p>';
            echo '<code style="word-break: break-all; display: block; background: #f5f5f5; padding: 8px; border-radius: 3px;">';
            echo esc_html($elid);
            echo '</code>';
            echo '<p style="margin-top: 10px;">';
            echo '<small>Last updated: ' . esc_html(get_post_meta($post->ID, '_elid_updated', true)) . '</small>';
            echo '</p>';
        } else {
            echo '<p><em>No ELID generated yet.</em></p>';
        }

        if ($embedding_available) {
            echo '<p><label>';
            echo '<input type="checkbox" name="regenerate_elid" value="1" />';
            echo ' Regenerate ELID on save';
            echo '</label></p>';
        }

        echo '</div>';
    }

    /**
     * Save post metadata
     */
    public static function save_post_meta($post_id)
    {
        // Verify nonce
        if (!isset($_POST['elid_meta_box_nonce']) ||
            !wp_verify_nonce($_POST['elid_meta_box_nonce'], 'elid_meta_box')) {
            return;
        }

        // Check autosave
        if (defined('DOING_AUTOSAVE') && DOING_AUTOSAVE) {
            return;
        }

        // Check permissions
        if (!current_user_can('edit_post', $post_id)) {
            return;
        }

        // Get post content
        $post = get_post($post_id);
        if (!$post) {
            return;
        }

        // Generate embedding from post content (this is a simplified example)
        // In production, you would call an actual embedding API (OpenAI, etc.)
        $embedding = self::generate_embedding_from_text($post->post_content);

        if ($embedding && (
            !get_post_meta($post_id, '_elid', true) ||
            isset($_POST['regenerate_elid'])
        )) {
            try {
                // Generate ELID
                $elid = Elid::encode($embedding, Elid::MINI128);

                // Save to post meta
                update_post_meta($post_id, '_elid', $elid);
                update_post_meta($post_id, '_elid_embedding_set', true);
                update_post_meta($post_id, '_elid_updated', current_time('mysql'));

            } catch (ElidException $e) {
                error_log('ELID generation failed for post ' . $post_id . ': ' . $e->getMessage());
            }
        }
    }

    /**
     * Register REST API routes
     */
    public static function register_rest_routes()
    {
        // Get similar posts by ELID
        register_rest_route('elid/v1', '/similar/(?P<post_id>\d+)', [
            'methods' => 'GET',
            'callback' => [self::class, 'get_similar_posts'],
            'permission_callback' => '__return_true',
            'args' => [
                'post_id' => [
                    'required' => true,
                    'validate_callback' => function($param) {
                        return is_numeric($param);
                    }
                ],
                'limit' => [
                    'default' => 5,
                    'validate_callback' => function($param) {
                        return is_numeric($param) && $param > 0 && $param <= 50;
                    }
                ]
            ]
        ]);

        // Encode custom embedding
        register_rest_route('elid/v1', '/encode', [
            'methods' => 'POST',
            'callback' => [self::class, 'encode_embedding'],
            'permission_callback' => function() {
                return current_user_can('edit_posts');
            }
        ]);
    }

    /**
     * Get similar posts based on Hamming distance
     */
    public static function get_similar_posts($request)
    {
        $post_id = $request['post_id'];
        $limit = $request['limit'] ?? 5;

        // Get source post ELID
        $source_elid = get_post_meta($post_id, '_elid', true);
        if (!$source_elid) {
            return new WP_Error('no_elid', 'Post does not have an ELID', ['status' => 404]);
        }

        // Find all posts with ELIDs
        global $wpdb;
        $posts_with_elids = $wpdb->get_results(
            "SELECT post_id, meta_value as elid
             FROM {$wpdb->postmeta}
             WHERE meta_key = '_elid'
             AND post_id != {$post_id}"
        );

        // Calculate Hamming distances
        $distances = [];
        foreach ($posts_with_elids as $row) {
            try {
                $distance = Elid::hammingDistance($source_elid, $row->elid);
                $distances[] = [
                    'post_id' => $row->post_id,
                    'distance' => $distance,
                ];
            } catch (ElidException $e) {
                continue; // Skip posts with incompatible ELIDs
            }
        }

        // Sort by distance and limit
        usort($distances, function($a, $b) {
            return $a['distance'] <=> $b['distance'];
        });
        $distances = array_slice($distances, 0, $limit);

        // Get full post data
        $similar_posts = [];
        foreach ($distances as $item) {
            $post = get_post($item['post_id']);
            if ($post) {
                $similar_posts[] = [
                    'id' => $post->ID,
                    'title' => $post->post_title,
                    'excerpt' => wp_trim_words($post->post_content, 30),
                    'permalink' => get_permalink($post->ID),
                    'distance' => $item['distance'],
                ];
            }
        }

        return [
            'source_post_id' => $post_id,
            'similar_posts' => $similar_posts,
            'total_compared' => count($posts_with_elids),
        ];
    }

    /**
     * REST API endpoint to encode custom embedding
     */
    public static function encode_embedding($request)
    {
        $params = $request->get_json_params();

        if (!isset($params['embedding']) || !is_array($params['embedding'])) {
            return new WP_Error('invalid_embedding', 'Embedding must be an array', ['status' => 400]);
        }

        $profile = $params['profile'] ?? Elid::MINI128;

        try {
            $elid = Elid::encode($params['embedding'], $profile);
            return [
                'success' => true,
                'elid' => $elid,
                'profile' => $profile,
            ];
        } catch (ElidException $e) {
            return new WP_Error('encoding_failed', $e->getMessage(), ['status' => 500]);
        }
    }

    /**
     * Add admin menu page
     */
    public static function add_admin_menu()
    {
        add_options_page(
            'ELID Vector Search Settings',
            'ELID Search',
            'manage_options',
            'elid-settings',
            [self::class, 'render_settings_page']
        );
    }

    /**
     * Render settings page
     */
    public static function render_settings_page()
    {
        if (!current_user_can('manage_options')) {
            return;
        }

        global $wpdb;
        $posts_with_elids = $wpdb->get_var(
            "SELECT COUNT(*) FROM {$wpdb->postmeta} WHERE meta_key = '_elid'"
        );

        echo '<div class="wrap">';
        echo '<h1>ELID Vector Search</h1>';
        echo '<div class="card">';
        echo '<h2>Statistics</h2>';
        echo '<p>Posts with ELIDs: <strong>' . esc_html($posts_with_elids) . '</strong></p>';
        echo '</div>';
        echo '<div class="card">';
        echo '<h2>Usage</h2>';
        echo '<p>ELIDs are automatically generated when you save posts. To find similar posts, use the REST API:</p>';
        echo '<code>GET /wp-json/elid/v1/similar/{post_id}?limit=5</code>';
        echo '</div>';
        echo '</div>';
    }

    /**
     * Generate embedding from text (simplified example)
     *
     * In production, you would call an actual embedding API like:
     * - OpenAI Embeddings API
     * - Sentence Transformers
     * - Custom ML model
     */
    private static function generate_embedding_from_text($text)
    {
        // This is a DUMMY implementation for demonstration
        // Replace with actual embedding generation

        // For now, generate a deterministic embedding based on text hash
        $hash = md5($text);
        $embedding = [];

        for ($i = 0; $i < 768; $i++) {
            // Use hash bytes to generate pseudo-random values
            $byte_pos = $i % 16;
            $byte_val = hexdec(substr($hash, $byte_pos * 2, 2));
            $embedding[] = ($byte_val / 255.0) * 2.0 - 1.0;
        }

        return $embedding;
    }
}

// Initialize plugin
add_action('plugins_loaded', [ELID_Vector_Search::class, 'init']);
