/**
 * Terminal-style chat client.
 * Orchestrates socket lifecycle, authentication, UI state and crypto.
 */

import { deriveKeyFromPassword, encryptWithKey, decryptWithKey } from "./crypto.js";
import { UIManager } from "./ui.js";
import { getCookie, setCookie, deleteCookie } from "./utils.js";

class TerminalChat {
  constructor() {
    /** @type {WebSocket|null} */
    this.socket = null;
    /** @type {string} */
    this.username = "";
    /** @type {boolean} */
    this.isConnected = false;
    /** @type {boolean} */
    this.isAuthenticated = false;

    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
    this.reconnectDelay = 3000; // ms

    /** @type {string} */
    this.wsUrl = this.getWebSocketUrl();

    this.messageHistory = [];
    this.historyIndex = -1;

    /** @type {CryptoKey|null} */
    this.encryptionKey = null;

    /** @type {Set<string>} */
    this.onlineUsers = new Set();

    /** @type {UIManager} */
    this.ui = new UIManager();

    this.attachEventListeners();
    this.connect();
    this.startTerminalEffects();
  }

  /**
   * Compute the WebSocket URL from runtime config or location.
   * @returns {string}
   */
  getWebSocketUrl() {
    if (window.__RUNTIME_CONFIG__ && window.__RUNTIME_CONFIG__.WS_URL) {
      return window.__RUNTIME_CONFIG__.WS_URL;
    }
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const host = window.location.hostname;
    return `${protocol}//${host}:9001`;
  }

  attachEventListeners() {
    this.ui.elements.passwordForm.addEventListener("submit", (e) => {
      e.preventDefault();
      this.authenticateUser();
    });

    this.ui.elements.usernameForm.addEventListener("submit", (e) => {
      e.preventDefault();
      this.setUsername();
    });

    this.ui.elements.messageForm.addEventListener("submit", (e) => {
      e.preventDefault();
      this.sendMessage();
    });

    this.ui.elements.closeError.addEventListener("click", () => {
      this.ui.hideError();
    });

    this.ui.elements.messageInput.addEventListener("keydown", (e) => {
      if (e.key === "Tab") {
        e.preventDefault();
        // Placeholder for future tab-completion
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        if (this.historyIndex < this.messageHistory.length - 1) {
          this.historyIndex++;
          this.ui.elements.messageInput.value = this.messageHistory[this.historyIndex];
        }
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        if (this.historyIndex > 0) {
          this.historyIndex--;
          this.ui.elements.messageInput.value = this.messageHistory[this.historyIndex];
        } else if (this.historyIndex <= 0) {
          this.historyIndex = -1;
          this.ui.elements.messageInput.value = "";
        }
      }
    });
  }

  /**
   * Attempt automatic authentication using a saved password cookie.
   * Initializes the encryption key locally and sends password to server.
   * @param {string} savedPassword
   */
  async autoAuthenticateFromCookie(savedPassword) {
    try {
      this.ui.showAuthLoading();
      this.encryptionKey = await deriveKeyFromPassword(savedPassword);
      this.socket?.send(
        JSON.stringify({
          password: savedPassword,
        })
      );
    } catch (error) {
      console.error("Auto-auth failed:", error);
      this.ui.hideAuthLoading();
      this.ui.showPasswordForm();
    }
  }

  startTerminalEffects() {
    setInterval(() => {
      if (Math.random() < 0.05) {
        const noise = document.createElement("span");
        noise.textContent = String.fromCharCode(Math.random() * 26 + 65);
        noise.className = "text-green-800 opacity-20 absolute";
        noise.style.left = Math.random() * 100 + "%";
        noise.style.top = Math.random() * 100 + "%";
        document.body.appendChild(noise);
        setTimeout(() => noise.remove(), 100);
      }
    }, 1000);
  }

  connect() {
    if (this.socket && this.socket.readyState === WebSocket.CONNECTING) {
      return;
    }

    this.ui.showLoading();
    this.ui.updateConnectionStatus("CONNECTING", "connecting");

    try {
      this.socket = new WebSocket(this.wsUrl);
      this.setupSocketHandlers();
    } catch (error) {
      console.error("Connection failed:", error);
      this.handleConnectionError();
    }
  }

  setupSocketHandlers() {
    if (!this.socket) return;

    this.socket.onopen = () => {
      this.isConnected = true;
      this.reconnectAttempts = 0;
      this.ui.updateConnectionStatus("CONNECTED", "connected");
      this.ui.hideLoading();

      if (!this.isAuthenticated) {
        const savedPassword = getCookie("chat_password");
        if (savedPassword) {
          // auto-authenticate from cookie
          this.autoAuthenticateFromCookie(savedPassword);
        } else {
          this.ui.showPasswordForm();
        }
      } else if (!this.username) {
        const savedUsername = getCookie("chat_username");
        if (savedUsername) {
          this.ui.elements.usernameInput.value = savedUsername;
          this.setUsername();
        } else {
          this.ui.showUsernameForm();
        }
      } else {
        this.ui.showChatInterface(this.username);
      }
    };

    this.socket.onmessage = async (event) => {
      try {
        const data = JSON.parse(event.data);
        await this.handleMessage(data);
      } catch (error) {
        console.error("Message parse error:", error);
        this.ui.displaySystemMessage(`[SYSTEM] Raw data: ${event.data}`);
      }
    };

    this.socket.onclose = (event) => {
      this.isConnected = false;
      this.isAuthenticated = false;

      if (event.wasClean) {
        this.ui.updateConnectionStatus("DISCONNECTED", "default");
      } else {
        this.handleConnectionError();
      }
    };

    this.socket.onerror = (error) => {
      console.error("Terminal error:", error);
      this.handleConnectionError();
    };
  }

  async handleMessage(data) {
    // Handle authentication responses
    if (data.success !== undefined) {
      this.ui.hideAuthLoading();
      if (data.success) {
        this.isAuthenticated = true;
        this.ui.elements.authError.classList.add("hidden");
        this.ui.displaySystemMessage(`[AUTH] ${data.message}`);
        // persist password for future auto-login
        const typedPassword = this.ui.elements.passwordInput.value.trim();
        if (typedPassword) {
          setCookie("chat_password", typedPassword, 30);
        }
        const savedUsername = getCookie("chat_username");
        if (savedUsername) {
          this.ui.elements.usernameInput.value = savedUsername;
          this.setUsername();
        } else {
          this.ui.showUsernameForm();
        }
      } else {
        // briefly show loading to avoid flash of previous form state
        this.ui.showPasswordForm();
        this.ui.elements.authError.classList.remove("hidden");
        this.ui.elements.passwordInput.value = "";
        // remove bad saved credentials
        deleteCookie("chat_password");
        this.encryptionKey = null;
        this.ui.elements.passwordInput.focus();
      }
      return;
    }

    // Decrypt encrypted messages, otherwise show plaintext
    if (data.encrypted_message && data.nonce) {
      try {
        if (!this.encryptionKey) {
          throw new Error("Encryption key not initialized");
        }
        const decryptedMessage = await decryptWithKey(
          this.encryptionKey,
          data.encrypted_message,
          data.nonce
        );

        if (data.message_type === "join") {
          this.ui.displaySystemMessage(`[JOIN] ${decryptedMessage}`);
          if (data.username) this.onlineUsers.add(data.username);
        } else if (data.message_type === "leave") {
          this.ui.displaySystemMessage(`[LEAVE] ${decryptedMessage}`);
          if (data.username) this.onlineUsers.delete(data.username);
        } else if (data.message_type === "message") {
          const decryptedData = { ...data, message: decryptedMessage };
          delete decryptedData.encrypted_message;
          delete decryptedData.nonce;
          this.ui.displayMessage(decryptedData, this.username);
          if (data.username) this.onlineUsers.add(data.username);
        }
      } catch (error) {
        console.error("Decryption failed:", error);
        this.ui.displaySystemMessage("[DECRYPTION FAILED]");
      }
    } else {
      if (data.message_type === "join" || data.message_type === "leave") {
        this.ui.displaySystemMessage(
          `[${data.message_type.toUpperCase()}] ${data.username} ${
            data.message_type === "join" ? "joined" : "left"
          } the chat`
        );
        if (data.message_type === "join" && data.username) this.onlineUsers.add(data.username);
        if (data.message_type === "leave" && data.username) this.onlineUsers.delete(data.username);
      } else {
        this.ui.displayMessage(data, this.username);
        if (data.username) this.onlineUsers.add(data.username);
      }
    }
  }

  async authenticateUser() {
    const password = this.ui.elements.passwordInput.value.trim();

    if (!password) {
      this.ui.showError("Password required");
      return;
    }
    if (!this.isConnected) {
      this.ui.showError("Terminal not connected");
      return;
    }

    this.ui.showAuthLoading();

    try {
      this.encryptionKey = await deriveKeyFromPassword(password);
    } catch (error) {
      console.error("Failed to initialize encryption:", error);
      this.ui.hideAuthLoading();
      this.ui.showError("Failed to initialize encryption");
      return;
    }

    try {
      this.socket?.send(
        JSON.stringify({
          password: password,
        })
      );
    } catch (error) {
      console.error("Auth failed:", error);
      this.ui.hideAuthLoading();
      this.ui.showError("Authentication failed");
    }
  }

  handleConnectionError() {
    this.isConnected = false;
    this.isAuthenticated = false;

    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = this.reconnectDelay * Math.pow(1.5, this.reconnectAttempts - 1);

      this.ui.updateConnectionStatus(
        `RECONNECTING (${this.reconnectAttempts}/${this.maxReconnectAttempts})`,
        "connecting"
      );

      setTimeout(() => this.connect(), delay);
    } else {
      this.ui.updateConnectionStatus("CONNECTION FAILED", "error");
      this.ui.showError("Terminal connection lost. Restart required.");
    }
  }

  setUsername() {
    const username = this.ui.elements.usernameInput.value.trim();

    if (!username) {
      this.ui.showError("Username required");
      return;
    }
    if (!this.isConnected || !this.isAuthenticated) {
      this.ui.showError("Terminal not authenticated");
      return;
    }
    if (username.length > 20) {
      this.ui.showError("Username too long (max 20 chars)");
      return;
    }
    if (!/^[a-zA-Z0-9_-]+$/.test(username)) {
      this.ui.showError("Invalid characters in username");
      return;
    }
    if (this.onlineUsers.has(username) && username !== this.username) {
      this.ui.showError("Username already taken. Choose another.");
      return;
    }

    this.username = username;
    setCookie("chat_username", this.username, 30);

    try {
      this.socket?.send(
        JSON.stringify({
          username: this.username,
          message_type: "join",
        })
      );
      this.ui.showChatInterface(this.username);
      this.onlineUsers.add(this.username);
    } catch (error) {
      console.error("Username set failed:", error);
      this.ui.showError("Username setting failed");
    }
  }

  async sendMessage() {
    const message = this.ui.elements.messageInput.value.trim();
    if (!message) return;

    this.messageHistory.unshift(message);
    this.historyIndex = -1;

    // Commands
    if (message.startsWith("/")) {
      this.handleCommand(message);
      this.ui.elements.messageInput.value = "";
      return;
    }

    if (!this.isConnected || !this.isAuthenticated) {
      this.ui.showError("Terminal not authenticated");
      return;
    }
    if (message.length > 500) {
      this.ui.showError("Message too long (max 500 chars)");
      return;
    }

    try {
      if (!this.encryptionKey) {
        throw new Error("Encryption key not initialized");
      }
      const encrypted = await encryptWithKey(this.encryptionKey, message);

      this.socket?.send(
        JSON.stringify({
          user_id: "", // Server sets this
          username: this.username,
          encrypted_message: encrypted.encrypted_message,
          nonce: encrypted.nonce,
          timestamp: new Date().toISOString(),
          message_type: "message",
        })
      );

      this.ui.elements.messageInput.value = "";
    } catch (error) {
      console.error("Send failed:", error);
      this.ui.showError("Message send failed");
    }
  }

  handleCommand(command) {
    const [cmd] = command.slice(1).split(" ");
    switch (cmd.toLowerCase()) {
      case "help":
        this.ui.displaySystemMessage("[HELP] Available commands: /help, /clear, /time, /ping, /list, /logout");
        break;
      case "clear":
        this.ui.elements.messagesContainer.innerHTML = "";
        break;
      case "time":
        this.ui.displaySystemMessage(`[TIME] ${new Date().toLocaleString()}`);
        break;
      case "ping":
        this.ui.displaySystemMessage("[PING] Connection active");
        break;
      case "list": {
        const users = Array.from(this.onlineUsers).sort((a, b) => a.localeCompare(b));
        if (users.length === 0) {
          this.ui.displaySystemMessage("[LIST] No users online yet");
        } else {
          this.ui.displaySystemMessage(`[LIST] ${users.length} online: ${users.join(", ")}`);
        }
        break;
      }
      case "logout":
        this.logout();
        break;
      default:
        this.ui.displaySystemMessage(`[ERROR] Unknown command: /${cmd}`);
    }
  }

  logout() {
    this.ui.displaySystemMessage("[SYSTEM] Logging out...");
    this.isAuthenticated = false;
    this.username = "";
    this.encryptionKey = null;
    this.onlineUsers.clear();
    // clear persisted credentials on explicit logout
    deleteCookie("chat_password");
    deleteCookie("chat_username");

    if (this.socket) this.socket.close();

    this.ui.elements.passwordInput.value = "";
    this.ui.elements.usernameInput.value = "";
    this.ui.elements.authError.classList.add("hidden");
    this.ui.elements.messagesContainer.innerHTML = "";

    setTimeout(() => {
      this.connect();
    }, 1000);
  }
}

// Entry point
window.addEventListener("DOMContentLoaded", () => {
  new TerminalChat();
});
