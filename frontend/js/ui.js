/**
 * UI manager encapsulates DOM lookups and view updates.
 * Keeps the rest of the app free of direct DOM manipulation.
 */

import { escapeHtml, formatTimestamp } from "./utils.js";

export class UIManager {
  constructor() {
    /** @type {Record<string, HTMLElement>} */
    this.elements = {
      connectionStatus: document.getElementById("connectionStatus"),
      passwordSection: document.getElementById("passwordSection"),
      passwordForm: document.getElementById("passwordForm"),
      passwordInput: document.getElementById("passwordInput"),
      passwordSubmit: document.getElementById("passwordSubmit"),
      authError: document.getElementById("authError"),
      authLoadingState: document.getElementById("authLoadingState"),
      usernameSection: document.getElementById("usernameSection"),
      usernameForm: document.getElementById("usernameForm"),
      usernameInput: document.getElementById("usernameInput"),
      chatInterface: document.getElementById("chatInterface"),
      loadingState: document.getElementById("loadingState"),
      messagesContainer: document.getElementById("messagesContainer"),
      messageForm: document.getElementById("messageForm"),
      messageInput: document.getElementById("messageInput"),
      promptPrefix: document.getElementById("promptPrefix"),
      errorToast: document.getElementById("errorToast"),
      errorMessage: document.getElementById("errorMessage"),
      closeError: document.getElementById("closeError"),
    };
    
    // Validate all required elements exist
    const missingElements = [];
    for (const [key, element] of Object.entries(this.elements)) {
      if (!element) {
        missingElements.push(key);
      }
    }
    if (missingElements.length > 0) {
      throw new Error(`Missing required DOM elements: ${missingElements.join(", ")}`);
    }
  }

  /**
   * Update the connection status indicator.
   * @param {string} message - Status text
   * @param {"connected"|"connecting"|"error"|"default"} type
   */
  updateConnectionStatus(message, type = "default") {
    const statusEl = this.elements.connectionStatus;
    const dot = statusEl.querySelector("span:first-child");
    const text = statusEl.querySelector("span:last-child");

    text.textContent = message;

    dot.className = `w-2 h-2 rounded-full mr-2 ${
      type === "connected"
        ? "bg-green-400"
        : type === "connecting"
        ? "bg-yellow-400 animate-pulse"
        : "bg-red-400 animate-pulse"
    }`;

    text.className =
      type === "connected"
        ? "text-green-400"
        : type === "connecting"
        ? "text-yellow-400"
        : "text-red-400";
  }

  showLoading() {
    this.elements.loadingState.classList.remove("hidden");
    this.elements.passwordSection.classList.add("hidden");
    this.elements.usernameSection.classList.add("hidden");
    this.elements.chatInterface.classList.add("hidden");
    this.elements.authLoadingState.classList.add("hidden");
  }

  hideLoading() {
    this.elements.loadingState.classList.add("hidden");
  }

  showAuthLoading() {
    this.elements.authLoadingState.classList.remove("hidden");
    this.elements.passwordSection.classList.add("hidden");
    this.elements.usernameSection.classList.add("hidden");
    this.elements.chatInterface.classList.add("hidden");
    this.elements.loadingState.classList.add("hidden");
    // disable inputs while loading
    this.elements.passwordInput.disabled = true;
    this.elements.passwordSubmit.disabled = true;
    this.elements.passwordSubmit.classList.add("opacity-60", "cursor-not-allowed");
  }

  hideAuthLoading() {
    this.elements.authLoadingState.classList.add("hidden");
    // re-enable inputs after loading
    this.elements.passwordInput.disabled = false;
    this.elements.passwordSubmit.disabled = false;
    this.elements.passwordSubmit.classList.remove("opacity-60", "cursor-not-allowed");
  }

  showPasswordForm() {
    this.elements.passwordSection.classList.remove("hidden");
    this.elements.usernameSection.classList.add("hidden");
    this.elements.chatInterface.classList.add("hidden");
    this.elements.authLoadingState.classList.add("hidden");
    this.elements.passwordInput.focus();
  }

  showUsernameForm() {
    this.elements.passwordSection.classList.add("hidden");
    this.elements.usernameSection.classList.remove("hidden");
    this.elements.chatInterface.classList.add("hidden");
    this.elements.authLoadingState.classList.add("hidden");
    this.elements.usernameInput.focus();
  }

  showChatInterface(username) {
    this.elements.passwordSection.classList.add("hidden");
    this.elements.usernameSection.classList.add("hidden");
    this.elements.chatInterface.classList.remove("hidden");
    this.elements.authLoadingState.classList.add("hidden");
    this.elements.promptPrefix.textContent = `${username}@chat:~$`;
    this.elements.messageInput.focus();
  }

  displayMessage(data, currentUsername) {
    const messageDiv = document.createElement("div");
    const timestamp = formatTimestamp(data.timestamp);
    const isOwnMessage = data.username === currentUsername;

    messageDiv.className = `mb-1 ${isOwnMessage ? "text-green-300" : "text-green-400"}`;

    const prefix = isOwnMessage ? ">" : "<";
    const userColor = isOwnMessage ? "text-green-300" : "text-cyan-400";

    const displayMessage = data.message || "[No message content]";

    messageDiv.innerHTML = `
      <span class="text-green-600">[${timestamp}]</span>
      <span class="text-yellow-400">${prefix}</span>
      <span class="${userColor}">${escapeHtml(data.username)}</span>
      <span class="text-green-600">:</span>
      <span class="ml-1">${escapeHtml(displayMessage)}</span>
    `;

    this.elements.messagesContainer.appendChild(messageDiv);
    this.scrollToBottom();
    this.limitMessages();
  }

  displaySystemMessage(message) {
    const messageDiv = document.createElement("div");
    messageDiv.className = "mb-1 text-yellow-400";
    messageDiv.textContent = message;

    this.elements.messagesContainer.appendChild(messageDiv);
    this.scrollToBottom();
    this.limitMessages();
  }

  scrollToBottom() {
    const container = this.elements.messagesContainer;
    // Use requestAnimationFrame to ensure DOM is updated before scrolling
    requestAnimationFrame(() => {
      // For mobile, ensure we scroll smoothly
      if (window.innerWidth <= 768) {
        container.scrollTo({
          top: container.scrollHeight,
          behavior: 'smooth'
        });
      } else {
        container.scrollTop = container.scrollHeight;
      }
    });
  }

  limitMessages() {
    const messages = this.elements.messagesContainer.children;
    if (messages.length > 200) {
      messages[0].remove();
    }
  }

  showError(message, isPersistent = false) {
    this.elements.errorMessage.textContent = message;
    this.elements.errorToast.classList.remove("hidden");
    if (!isPersistent) {
      setTimeout(() => this.hideError(), 5000);
    }
  }

  hideError() {
    this.elements.errorToast.classList.add("hidden");
  }
}
