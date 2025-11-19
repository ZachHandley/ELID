#ifndef ELID_H
#define ELID_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Represents a match result with index and score
 */
typedef struct ElidMatch {
  /**
   * Index of the matched string in the candidates array
   */
  uintptr_t index;
  /**
   * Similarity score (0.0 to 1.0)
   */
  double score;
} ElidMatch;

/**
 * Represents an array of match results
 */
typedef struct ElidMatchArray {
  /**
   * Pointer to the array of matches
   */
  struct ElidMatch *matches;
  /**
   * Number of matches in the array
   */
  uintptr_t length;
} ElidMatchArray;

/**
 * Compute the Levenshtein distance between two strings.
 *
 * Returns the minimum number of single-character edits needed to transform one string into another.
 *
 * # Safety
 *
 * Both `a` and `b` must be valid, null-terminated UTF-8 strings.
 * Returns 0 if either pointer is NULL.
 */
uintptr_t elid_levenshtein(const char *a, const char *b);

/**
 * Compute the normalized Levenshtein similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 *
 * # Safety
 *
 * Both `a` and `b` must be valid, null-terminated UTF-8 strings.
 * Returns 0.0 if either pointer is NULL.
 */
double elid_normalized_levenshtein(const char *a, const char *b);

/**
 * Compute the Jaro similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 *
 * # Safety
 *
 * Both `a` and `b` must be valid, null-terminated UTF-8 strings.
 * Returns 0.0 if either pointer is NULL.
 */
double elid_jaro(const char *a, const char *b);

/**
 * Compute the Jaro-Winkler similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 *
 * # Safety
 *
 * Both `a` and `b` must be valid, null-terminated UTF-8 strings.
 * Returns 0.0 if either pointer is NULL.
 */
double elid_jaro_winkler(const char *a, const char *b);

/**
 * Compute the Hamming distance between two strings.
 *
 * Returns the number of positions at which the characters differ.
 * Returns -1 if strings have different lengths or if either pointer is NULL.
 *
 * # Safety
 *
 * Both `a` and `b` must be valid, null-terminated UTF-8 strings.
 */
int64_t elid_hamming(const char *a, const char *b);

/**
 * Compute the OSA (Optimal String Alignment) distance between two strings.
 *
 * Similar to Levenshtein but also considers transpositions as a single operation.
 *
 * # Safety
 *
 * Both `a` and `b` must be valid, null-terminated UTF-8 strings.
 * Returns 0 if either pointer is NULL.
 */
uintptr_t elid_osa_distance(const char *a, const char *b);

/**
 * Compute the best matching similarity between two strings.
 *
 * Runs multiple algorithms and returns the highest score.
 *
 * # Safety
 *
 * Both `a` and `b` must be valid, null-terminated UTF-8 strings.
 * Returns 0.0 if either pointer is NULL.
 */
double elid_best_match(const char *a, const char *b);

/**
 * Compute the SimHash fingerprint of a string.
 *
 * Returns a 64-bit hash where similar strings produce similar numbers.
 *
 * # Safety
 *
 * `text` must be a valid, null-terminated UTF-8 string.
 * Returns 0 if pointer is NULL.
 */
uint64_t elid_simhash(const char *text);

/**
 * Compute the Hamming distance between two SimHash values.
 *
 * Returns the number of differing bits. Lower values = higher similarity.
 */
uint32_t elid_simhash_distance(uint64_t hash1, uint64_t hash2);

/**
 * Compute the normalized SimHash similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 *
 * # Safety
 *
 * Both `a` and `b` must be valid, null-terminated UTF-8 strings.
 * Returns 0.0 if either pointer is NULL.
 */
double elid_simhash_similarity(const char *a, const char *b);

/**
 * Free a string allocated by Rust.
 *
 * This must be called on all strings returned by ELID functions to prevent memory leaks.
 *
 * # Safety
 *
 * `s` must be a string previously returned by an ELID function, or NULL.
 * Do not call this function twice on the same pointer.
 */
void elid_free_string(char *s);

/**
 * Free a match array allocated by Rust.
 *
 * This must be called on all match arrays returned by ELID functions to prevent memory leaks.
 *
 * # Safety
 *
 * `array` must be a match array previously returned by an ELID function.
 * Do not call this function twice on the same pointer.
 */
void elid_free_match_array(struct ElidMatchArray array);

/**
 * Get the library version as a static string.
 *
 * The returned string does not need to be freed.
 */
const char *elid_version(void);

#endif /* ELID_H */
