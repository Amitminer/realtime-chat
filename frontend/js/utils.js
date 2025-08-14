/**
 * Utility helpers for formatting, encoding and safe HTML operations.
 * These are browser-only helpers with no external dependencies.
 */

/**
 * Convert a Base64 string to an ArrayBuffer.
 * @param {string} base64 - Base64 encoded string
 * @returns {ArrayBuffer}
 */
export function base64ToArrayBuffer(base64) {
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes.buffer;
}

/**
 * Convert an ArrayBuffer to a Base64 string.
 * @param {ArrayBuffer} buffer - Raw binary data
 * @returns {string}
 */
export function arrayBufferToBase64(buffer) {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

/**
 * Escape potentially unsafe HTML text for insertion.
 * @param {string} text - Untrusted text
 * @returns {string}
 */
export function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

/**
 * Format an ISO timestamp into HH:mm:ss (24h) local time.
 * @param {string|number|Date} timestamp
 * @returns {string}
 */
export function formatTimestamp(timestamp) {
  try {
    return new Date(timestamp).toLocaleTimeString("en-US", {
      hour12: false,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return new Date().toLocaleTimeString("en-US", {
      hour12: false,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }
}

/**
 * Set a cookie with path=/ and optional expiration in days.
 * @param {string} name
 * @param {string} value
 * @param {number} days
 */
export function setCookie(name, value, days = 30) {
  const maxAge = days * 24 * 60 * 60;
  document.cookie = `${encodeURIComponent(name)}=${encodeURIComponent(value)}; max-age=${maxAge}; path=/`;
}

/**
 * Get a cookie by name.
 * @param {string} name
 * @returns {string|undefined}
 */
export function getCookie(name) {
  const key = encodeURIComponent(name) + "=";
  const parts = document.cookie.split("; ");
  for (const part of parts) {
    if (part.startsWith(key)) {
      return decodeURIComponent(part.substring(key.length));
    }
  }
  return undefined;
}

/**
 * Delete a cookie by setting it expired.
 * @param {string} name
 */
export function deleteCookie(name) {
  document.cookie = `${encodeURIComponent(name)}=; max-age=0; path=/`;
}
