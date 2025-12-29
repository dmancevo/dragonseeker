/**
 * Security utilities for safe HTML rendering
 * Prevents XSS attacks by properly escaping user-generated content
 */

/**
 * Escapes HTML special characters to prevent XSS attacks
 *
 * @param {string} unsafe - The untrusted string to escape
 * @returns {string} - The escaped string safe for HTML insertion
 *
 * @example
 * escapeHtml('<script>alert("xss")</script>')
 * // Returns: '&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;'
 */
function escapeHtml(unsafe) {
    if (typeof unsafe !== 'string') {
        return '';
    }

    return unsafe
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}
